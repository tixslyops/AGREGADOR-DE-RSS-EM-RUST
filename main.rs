mod db;
mod fetcher;
mod models;
mod parser;

use anyhow::Result;
use clap::{Parser, Subcommand};
use db::Database;
use fetcher::Fetcher;

#[derive(Parser)]
#[command(name = "RSS Aggregator")]
#[command(about = "Agregador concorrente de feeds RSS/Atom em Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { url: String },
    Remove { id: i64 },
    ListFeeds,
    Sync,
    Articles {
        #[arg(short, long)]
        unread: bool,
        #[arg(short, long)]
        feed_id: Option<i64>,
        #[arg(short, long)]
        search: Option<String>,
    },
    Read { id: i64 },
    Fav { id: i64 },
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = Database::init("rss_data.db")?;
    let fetcher = Fetcher::new(db.clone(), 5); 

    let cli = Cli::parse();

    match cli.command {
        Commands::Add { url } => {
            let id = db.add_feed(&url)?;
            println!("Feed cadastrado com sucesso [ID: {}]!", id);
            if let Ok(feeds) = db.list_feeds() {
                if let Some(f) = feeds.into_iter().find(|item| item.id == id) {
                    println!("A descarregar o novo feed...");
                    let _ = fetcher.fetch_feed(f).await;
                }
            }
        }
        Commands::Remove { id } => {
            db.remove_feed(id)?;
            println!("Feed ID {} removido com sucesso.", id);
        }
        Commands::ListFeeds => {
            let feeds = db.list_feeds()?;
            println!("{:<5} | {:<30} | {:<40} | Failures", "ID", "Título", "URL");
            println!("{:-<90}", "");
            for f in feeds {
                println!(
                    "{:<5} | {:<30} | {:<40} | {}",
                    f.id,
                    f.title.unwrap_or_else(|| "Sem título".into()),
                    f.url,
                    f.failure_count
                );
            }
        }
        Commands::Sync => {
            println!("A atualizar feeds concorrentemente...");
            let count = fetcher.fetch_all().await?;
            println!("Atualização concluída! {} novos artigos recebidos.", count);
        }
        Commands::Articles {
            unread,
            feed_id,
            search,
        } => {
            let articles = db.get_articles(unread, feed_id, search.as_deref())?;
            println!("{:<5} | {:<6} | {:<6} | Título e Link", "ID", "Lido", "Fav");
            println!("{:-<80}", "");
            for a in articles {
                let read_flag = if a.is_read { "Sim" } else { "Não" };
                let fav_flag = if a.is_favorite { "★" } else { " " };
                println!("{:<5} | {:<6} | {:<6} | {}", a.id, read_flag, fav_flag, a.title);
                println!("      Link: {}", a.link);
                if let Some(desc) = a.description {
                    let clean_desc = if desc.len() > 100 { format!("{}...", &desc[..100]) } else { desc };
                    println!("      Resumo: {}", clean_desc.replace('\n', " "));
                }
                println!();
            }
        }
        Commands::Read { id } => {
            db.set_read_status(id, true)?;
            println!("Artigo {} marcado como lido.", id);
        }
        Commands::Fav { id } => {
            db.set_favorite_status(id, true)?;
            println!("Artigo {} marcado como favorito.", id);
        }
    }

    Ok(())
}