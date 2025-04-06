use dialoguer;
use reqwest;
use scraper::{Html, Selector};
use std::error::Error;
use std::io;
use std::io::Write;
mod ui;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    print!("Enter the name of the novel: ");
    io::stdout().flush().unwrap();

    let mut url = String::new();
    io::stdin()
        .read_line(&mut url)
        .expect("failed to read line");

    let name_split = url.split_whitespace();
    let mut link = String::from("https://novelbin.me/search?keyword=");
    for word in name_split {
        link += word;
        link += "+";
    }

    let mut titles = Vec::new();
    let mut title_links = Vec::new();

    let body = reqwest::get(link).await?.text().await?;
    let doc = Html::parse_document(&body);

    let selector = Selector::parse("h3.novel-title a").unwrap();
    for heading in doc.select(&selector) {
        if let Some(link) = heading.value().attr("href") {
            titles.push(heading.text().collect::<String>());
            let pre_title = heading.text().collect::<String>();
            let mut novel_id = pre_title.to_lowercase();

            novel_id.retain(|c| !matches!(c, '(' | ')' | ':'));
            novel_id = novel_id.replace(" ", "-");
            title_links.push(format!(
                "https://novelbin.me/ajax/chapter-archive?novelId={}",
                novel_id
            ));
        }
    }

    let selection = dialoguer::FuzzySelect::new()
        .with_prompt("Results ")
        .items(&titles)
        .interact()
        .unwrap();

    let c_body = reqwest::get(&title_links[selection]).await?.text().await?;
    let c_doc = Html::parse_document(&c_body);
    let c_selector = Selector::parse(".nchr-text.chapter-title").unwrap();

    let mut chapters: Vec<String> = Vec::new();
    let mut chapter_links: Vec<String> = Vec::new();

    for chapter in c_doc.select(&c_selector) {
        let chapter_text = chapter.text().collect::<String>().trim().to_string();
        chapters.push(chapter_text);

        if let Some(anchor) = chapter.parent() {
            if let Some(href) = anchor.value().as_element().and_then(|e| e.attr("href")) {
                chapter_links.push(href.to_string());
            }
        }
    }

    let c_selection = dialoguer::FuzzySelect::new()
        .with_prompt("Results ")
        .items(&chapters)
        .interact()
        .unwrap();
    println!("{}", chapters[c_selection]);
    println!("{}", chapter_links[c_selection]);

    let chapter = reqwest::get(&chapter_links[c_selection])
        .await?
        .text()
        .await?;

    let chapter_doc = Html::parse_document(&chapter);
    let base_selector = Selector::parse("p").unwrap();
    let mut content = Vec::new();
    for para in chapter_doc.select(&base_selector) {
        let text = para.text().collect::<String>().trim().to_string();
        content.push(text);
    }
    content.pop();
    content.pop();
    content.pop();
    ui::ui_run();
    Ok(())
}
