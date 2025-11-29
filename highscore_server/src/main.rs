use axum::{Json, Router, extract::State, response::Html, routing::{get, post}};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/scoreSubmit", post(score_submit))
        .route("/", get(high_scores_html))
        .route("/highScores", get(high_scores_json))
        .with_state(Arc::new(Mutex::new(HighScoreTable::new())));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct HighScoreEntry {
    name: String,
    score: u32,
}

async fn score_submit(
    State(table): State<Arc<Mutex<HighScoreTable>>>,
    high_score: Json<HighScoreEntry>,
) {
    let mut lock = table.lock().await;
    lock.add_entry(HighScoreEntry {
        name: high_score.name.clone(),
        score: high_score.score,
    });
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct HighScoreTable {
    entries: Vec<HighScoreEntry>,
}

impl HighScoreTable {
    fn new() -> Self {
        if std::path::Path::new("high_socres.json").exists() {
            let file = std::fs::File::open("high_scores.json").unwrap();
            serde_json::from_reader(file).unwrap()
        } else {
            Self {
                entries: Vec::new(),
            }
        }
    }

    fn add_entry(&mut self, entry: HighScoreEntry) {
        self.entries.push(entry);
        self.entries.sort_by(|a, b| b.score.cmp(&a.score));
        self.entries.truncate(10);
        self.save();
    }

    fn save(&self) {
        let file = std::fs::File::create("high_scores.json").unwrap();
        serde_json::to_writer(file, self).unwrap();
    }
}

async fn high_scores_html(State(table): State<Arc<Mutex<HighScoreTable>>>) -> Html<String> {
    let mut html = String::from("<h1>High Scores</h1>");
    html.push_str("<table>");
    html.push_str("<tr><th>Name</th><th>Score</th></tr>");
    for entry in &table.lock().await.entries {
        html.push_str("<tr>");
        html.push_str("<td>");
        html.push_str(&entry.name);
        html.push_str("</td>");
        html.push_str("<td>");
        html.push_str(&entry.score.to_string());
        html.push_str("</td>");
        html.push_str("</tr>");
        html.push_str("</table>");
    }
    Html(html)
}

async fn high_scores_json(State(table): State<Arc<Mutex<HighScoreTable>>>) -> Json<HighScoreTable> {
    let lock = table.lock().await;
    let table = lock.clone();
    Json(table)
}
