#[macro_use]
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate simple_logger;

extern crate regex;
extern crate reqwest;
extern crate scraper;

use lambda::error::HandlerError;
use std::collections::HashMap;
use std::env;
use std::error::Error;

use regex::Regex;
use scraper::{Html, Selector};

#[derive(Deserialize, Clone)]
struct CustomEvent {
    body: String,
}

#[derive(Serialize, Clone)]
struct Body {
    response_type: String,
    text: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CustomOutput {
    is_base64_encoded: bool,
    status_code: i32,
    headers: HashMap<(), ()>,
    body: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info)?;

    let mut argument = env::args();
    match argument.len() {
        1 => {
            lambda!(lambda_handler);
        }
        2 => {
            cli_handler(argument.nth(1))?;
        }
        _ => {
            error!("too many arguments. usage: cargo run <month number>");
        }
    }
    Ok(())
}

fn cli_handler(arg: Option<String>) -> Result<(), Box<dyn Error>> {
    if let Some(month) = arg {
        match month.parse::<i32>() {
            Ok(month_num @ 1..=12) => {
                let res = sukkirisu(month_num)?;
                println!("{}", res);
            }
            Ok(_) => {
                error!("month \"{}\" should be in the range of 1 to 12", month);
            }
            Err(err) => {
                error!("month \"{}\" is not a number", month);
                return Err(Box::new(err));
            }
        }
    }
    Ok(())
}

fn lambda_handler(e: CustomEvent, c: lambda::Context) -> Result<CustomOutput, HandlerError> {
    if e.body == "" {
        error!("Empty body: request_id {}", c.aws_request_id);
        return Err(c.new_error("Empty body"));
    }
    let data = e.body.split("&").collect::<Vec<&str>>();
    let month = data
        .iter()
        .find(|&&x| x.contains("text="))
        .unwrap()
        .split("=")
        .collect::<Vec<&str>>()[1];
    if month == "" {
        error!("Empty month: request_id {}", c.aws_request_id);
        return Err(c.new_error("Empty month"));
    }
    match month.parse::<i32>() {
        Ok(month_num @ 1..=12) => match sukkirisu(month_num) {
            Ok(res) => Ok(CustomOutput {
                is_base64_encoded: false,
                status_code: 200,
                headers: HashMap::new(),
                body: serde_json::to_string(&Body {
                    response_type: String::from("in_channel"),
                    text: res,
                })
                .unwrap(),
            }),
            Err(err) => {
                error!(
                    "Failed to get sukkirisu rank of request {}",
                    c.aws_request_id
                );
                return Err(c.new_error(&err.to_string()));
            }
        },
        Ok(_) => {
            error!(
                "month \"{}\" should be in the range of 1 to 12: request_id {} ",
                month, c.aws_request_id
            );
            return Err(c.new_error("Invalid month"));
        }
        Err(err) => {
            error!(
                "month \"{}\" is not a number: request_id {}",
                month, c.aws_request_id
            );
            return Err(c.new_error(&format!("Invalid month: {}", err.to_string())));
        }
    }
}

#[tokio::main]
async fn sukkirisu(month: i32) -> Result<String, Box<dyn std::error::Error>> {
    let url = "http://www.ntv.co.jp/sukkiri/sukkirisu/index.html";

    /***** GETリクエスト *****/
    let res = reqwest::get(url).await?;
    let body = res.text().await?;

    /***** htmlのパース *****/
    let document = Html::parse_document(&body);

    // 1位から12位の月を記録する配列
    let mut month_list: Vec<i32> = Vec::new();
    let mut desc_list: Vec<&str> = Vec::new();
    let mut color_list: Vec<&str> = Vec::new();
    // サイトで表示される順番
    let ranking = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 1];

    let row1_list = document
        .select(&Selector::parse(r#"div[class="row1"]"#).unwrap())
        .collect::<Vec<_>>();
    for row1 in row1_list {
        let month_num: i32 = row1
            .select(&Selector::parse(r#"p[class="month"]"#).unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>()[0]
            .parse::<i32>()?;
        month_list.push(month_num);
    }
    // 探している月が何番目か
    let idx: usize = month_list.iter().position(|&x| x == month).unwrap();

    /***** 順位 *****/
    let rank: String;
    match ranking[idx] {
        r @ 2..=6 => {
            rank = format!("スッキリす🍀 {}位", r);
        }
        r @ 7..=11 => {
            rank = format!("まあまあスッキリす🍥 {}位", r);
        }
        1 => {
            rank = String::from("✨✨超スッキリす！！✨✨");
        }
        12 => {
            rank = String::from("ガッカリす...💧");
        }
        _ => {
            unreachable!();
        }
    }

    /***** 説明文, ラッキーカラー *****/
    let row2_list = document
        .select(&Selector::parse(r#"div[class="row2"]"#).unwrap())
        .collect::<Vec<_>>();
    for row2 in row2_list {
        let description: &str = row2
            .select(&Selector::parse("p").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>()[0];
        desc_list.push(description);
        let color: &str = row2
            .select(&Selector::parse("div").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>()[0];
        color_list.push(color);
    }

    /***** 更新日 *****/
    let span_date = document
        .select(&Selector::parse(r#"p[class="date"]"#).unwrap())
        .next()
        .unwrap();
    let span_date_txt = span_date.text().collect::<Vec<_>>()[0].trim();
    let re = Regex::new(r"\d+").unwrap();
    let mut cap = re.captures_iter(span_date_txt);
    let modified_date = format!("{}/{}", &cap.next().unwrap()[0], &cap.next().unwrap()[0]);

    Ok(format!(
        "{}月: {}\n{}\nラッキーカラー: {}\n更新日: {}",
        month, rank, desc_list[idx], color_list[idx], modified_date
    ))
}
