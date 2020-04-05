extern crate reqwest;
extern crate scraper;
extern crate regex;
use scraper::{Html, Selector};
use regex::Regex;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "http://www.ntv.co.jp/sukkiri/sukkirisu/index.html";

    /***** コマンドライン引数を読み込み *****/
    let args: Vec<String> = env::args().collect();
    let month: i32 = args[1].parse()?;

    /***** GETリクエスト *****/
    let res = reqwest::get(url).await?;
    let body = res.text().await?;

    /***** htmlのパース *****/
    let document = Html::parse_document(&body);
    let query = format!(r#"div[id="month{}"]"#, month);
    let div_month = document.select(&Selector::parse(&query).unwrap())
                    .next().unwrap();

    /***** 順位 *****/
    let div_month_class = div_month.value().attr("class").unwrap();
    let p_rank_selector = Selector::parse(r#"p[class="rankTxt"]"#).unwrap();
    let rank: String;
    if div_month_class.contains("type1") {
        rank = String::from("✨✨超スッキリす！！✨✨");
    } else if div_month_class.contains("type2") {
        let p_rank = div_month.select(&p_rank_selector)
                     .next().unwrap();
        let p_rank_txt = p_rank.text().collect::<Vec<_>>()[0];
        rank = format!("スッキリす🍀 {}", p_rank_txt);
    } else if div_month_class.contains("type3") {
        let p_rank = div_month.select(&p_rank_selector)
                     .next().unwrap();
        let p_rank_txt = p_rank.text().collect::<Vec<_>>()[0];
        rank = format!("まあまあスッキリす🍥 {}", p_rank_txt);
    //} else if div_month_class.contains("type4") {
    } else {
        rank = String::from("ガッカリす...💧");
    }

    /***** 説明文 *****/
    // 複数あるpタグの最後の要素をlast()で取得
    let p_description = div_month.select(&Selector::parse("p").unwrap())
                        .last().unwrap();
    let p_description_txt = p_description.text().collect::<Vec<_>>()[0];

    /***** ラッキーカラー *****/
    let div_color = div_month
                    .select(&Selector::parse(r#"div[id="color"]"#).unwrap())
                    .next().unwrap();
    let div_color_txt = div_color.text().collect::<Vec<_>>()[0];

    /***** 更新日 *****/
    let span_date = document.select(&Selector::parse(r#"span[class="date"]"#).unwrap())
                    .next().unwrap();
    let span_date_txt = span_date.text().collect::<Vec<_>>()[0].trim();
    let re = Regex::new(r"\d+").unwrap();
    let mut cap = re.captures_iter(span_date_txt);
    let modified_date = format!("{}/{}",
                                &cap.next().unwrap()[0],
                                &cap.next().unwrap()[0]);

    println!("{}月: {}\n{}\nラッキーカラー: {}\n更新日: {}",
             month, rank, p_description_txt, div_color_txt, modified_date);

    Ok(())
}
