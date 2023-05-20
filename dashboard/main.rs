use std::{io, path::Path, fs::File, cell::Cell, error::Error, ops::Add};

use crossterm::{execute, terminal::{ClearType, EnterAlternateScreen, Clear, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode}, event::{DisableMouseCapture, Event, self}};
use futures::FutureExt;
use reqwest::{Response, Client};
use tui::{backend::CrosstermBackend, Terminal, widgets::{Block, Borders, ListItem, Row, Table}, text::{Spans, Span}, style::{Style, Modifier}, layout::{Constraint, Layout, Direction}, symbols::block};

pub struct GlobalState {
  nodes: Vec<Node>,
  leader: i32,
}

#[derive(Debug)]
pub struct Node {
  name: String,
  address: String,
  is_tracked: bool,
}

const DEFAULT_CSV: &str = "nodes.csv";

#[tokio::main]
async fn main() -> Result<(), io::Error>{
  execute!(io::stdout(), Clear(ClearType::All), EnterAlternateScreen, DisableMouseCapture)?;
  let mut stdout = io::stdout();
	let backend = CrosstermBackend::new(&mut stdout);
	let mut terminal: Terminal<CrosstermBackend<&mut io::Stdout>> = Terminal::new(backend)?;
  
  let mut state = GlobalState {
    nodes: Vec::new(),
    leader: -1,
  };
  
  enable_raw_mode()?;
  let file_path = Path::new(DEFAULT_CSV);
  let file = match File::open(file_path) {
    Ok(file) => file,
    Err(_) => {
      println!("File not found, creating new file");
      File::create(file_path).unwrap()
    }
  };
  let mut reader = csv::Reader::from_reader(file);
  for result in reader.records() {
    match result {
      Ok(record) => {
        let name = record.get(0).unwrap().to_string();
        let address = record.get(1).unwrap().to_string();
        let is_tracked = record.get(2).unwrap().parse::<i32>().unwrap() == 1;
        state.nodes.push(Node {
          name,
          address,
          is_tracked,
        });
      },
      _ => {
        println!("Error reading file");
      }
    }
  }


  loop {
    let mut rows: Vec<Row> = Vec::new();
    for a in &state.nodes {
      rows.push(node_row(&a).await);
    }
    terminal.draw(|f| {
      let size = f.size();
      let divs = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

      let menu_block = Block::default()
        .title("Menu")
        .borders(Borders::ALL);
      f.render_widget(menu_block, divs[0]);


      let block = Block::default()
        .title("Node lists")
        .borders(Borders::ALL);      
      let headers = Row::new(vec![
        Span::styled("Node Name", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Address"),
        Span::raw("Status"),
      ]).style(tui::style::Style::default().fg(tui::style::Color::Yellow));
      let table = Table::new(rows)
        .block(block)
        .header(headers)
        .widths(&[Constraint::Percentage(25), Constraint::Percentage(50), Constraint::Percentage(25)]);
      
      f.render_widget(table, divs[1]);
    })?;

    if let Event::Key(key) = event::read()? {
      match key.code {
        event::KeyCode::Char('q') => {
          break;
        },
        _ => {}
      }
    }
  }

  disable_raw_mode()?;
	execute!(
		terminal.backend_mut(),
    Clear(ClearType::All),
		LeaveAlternateScreen,
		DisableMouseCapture
	)?;
  Ok(())
}

async fn node_row(node: &Node) -> Row<'static> {
  let response = get(node.address.clone(), "/ok".to_string()).await.unwrap();
  return Row::new(vec![
    Span::styled(node.name.clone(), Style::default().add_modifier(Modifier::BOLD)),
    Span::raw(node.address.clone()),
    Span::raw(response.to_string()),
  ]);
}

async fn get(address: String, path: String) -> Result<bool, Box<dyn Error>> {
  let client = Client::new();
  let url = format!("{}{}", address, path);
  match client.get(url).send().await {
    Ok(response) => Ok(true),
    Err(err) => {
      println!("Error: {:?}", err);
      Ok(false)
    },
  }
}