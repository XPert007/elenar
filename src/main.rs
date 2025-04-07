use dialoguer::FuzzySelect;
use reqwest;
use scraper::{Html, Selector};
use std::error::Error;
use std::io::{self, Write};
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let novel_name = prompt_novel_name()?;

    let search_url = build_search_url(&novel_name);

    let body = reqwest::get(&search_url).await?.text().await?;
    let (titles, title_links) = parse_search_results(&body);

    let novel_index = prompt_selection("Select a novel:", &titles)?;

    let archive_body = reqwest::get(&title_links[novel_index])
        .await?
        .text()
        .await?;
    let (chapters, chapter_links) = parse_chapters(&archive_body);

    let chapter_index = prompt_selection("Select a chapter:", &chapters)?;
    println!("Selected chapter: {}", chapters[chapter_index]);

    let chapter_body = reqwest::get(&chapter_links[chapter_index])
        .await?
        .text()
        .await?;
    let content = parse_chapter_content(&chapter_body);

    let title_parts: Vec<&str> = chapters[chapter_index].split_whitespace().collect();
    let subtitle = title_parts.get(1).unwrap_or(&"");
    let chapter_title = if title_parts.len() > 2 {
        title_parts[2..].join(" ")
    } else {
        String::new()
    };

    ui::run_ui(&titles[novel_index], subtitle, &chapter_title, content).unwrap();

    Ok(())
}

fn prompt_novel_name() -> Result<String, Box<dyn Error>> {
    print!("Enter the name of the novel: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn build_search_url(novel_name: &str) -> String {
    let query: String = novel_name
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("+");
    format!("https://novelbin.me/search?keyword={}", query)
}

fn parse_search_results(body: &str) -> (Vec<String>, Vec<String>) {
    let mut titles = Vec::new();
    let mut title_links = Vec::new();
    let document = Html::parse_document(body);
    let selector = Selector::parse("h3.novel-title a").unwrap();

    for element in document.select(&selector) {
        let title = element.text().collect::<String>().trim().to_string();
        if title.is_empty() {
            continue;
        }
        titles.push(title.clone());

        let mut novel_id = title.to_lowercase();
        novel_id.retain(|c| !matches!(c, '(' | ')' | ':'));
        novel_id = novel_id.replace(" ", "-");
        let chapter_archive_url = format!(
            "https://novelbin.me/ajax/chapter-archive?novelId={}",
            novel_id
        );
        title_links.push(chapter_archive_url);
    }
    (titles, title_links)
}

fn parse_chapters(body: &str) -> (Vec<String>, Vec<String>) {
    let mut chapters = Vec::new();
    let mut chapter_links = Vec::new();
    let document = Html::parse_document(body);
    let selector = Selector::parse(".nchr-text.chapter-title").unwrap();

    for chapter_element in document.select(&selector) {
        let chapter_title = chapter_element
            .text()
            .collect::<String>()
            .trim()
            .to_string();
        if chapter_title.is_empty() {
            continue;
        }
        chapters.push(chapter_title);

        if let Some(anchor) = chapter_element.parent() {
            if let Some(href) = anchor.value().as_element().and_then(|e| e.attr("href")) {
                chapter_links.push(href.to_string());
            }
        }
    }
    (chapters, chapter_links)
}

fn parse_chapter_content(body: &str) -> Vec<String> {
    let document = Html::parse_document(body);
    let selector = Selector::parse("p").unwrap();
    let mut paragraphs: Vec<String> = document
        .select(&selector)
        .map(|p| p.text().collect::<String>().trim().to_string())
        .collect();

    for _ in 0..3 {
        paragraphs.pop();
    }
    paragraphs
}

fn prompt_selection(prompt: &str, items: &[String]) -> Result<usize, Box<dyn Error>> {
    let selection = FuzzySelect::new()
        .with_prompt(prompt)
        .items(items)
        .interact()?;
    Ok(selection)
}
