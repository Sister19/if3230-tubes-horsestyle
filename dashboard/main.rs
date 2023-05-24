use std::time::SystemTime;
use std::{io, path::Path, fs::File, cell::Cell, error::Error, ops::Add};
use std::io::Stdout;

use actix_web::cookie::time::Duration;
use crossterm::{execute, terminal::{ClearType, EnterAlternateScreen, Clear, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode}, event::{DisableMouseCapture, Event, self}};
use futures::FutureExt;
use reqwest::{Response, Client};
use tui::{backend::CrosstermBackend, Terminal, widgets::{Block, Borders, ListItem, Row, Table, TableState}, text::{Spans, Span}, style::{Style, Modifier}, layout::{Constraint, Layout, Direction}, symbols::block, Frame};
use tui::style::Color;
use tui::symbols::DOT;
use tui::widgets::{Tabs, Paragraph, Wrap};
pub const REQUEST_LOG_ROUTE: &str = "/requestLog";
enum Pages {
  NodeList,
  NodeDetail,
  AddNode
}

pub struct GlobalState {
  nodes: Vec<Node>,
  leader: i32,
  current_page: Pages,
  table_state: TableState,
  selected_node: Option<Node>,
  last_updated: SystemTime
}

#[derive(Debug, Clone)]
pub struct Node {
  name: String,
  address: String,
  is_tracked: bool,
  log: String,
  status: bool
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
    current_page: Pages::NodeList,
    table_state: TableState::default(),
    selected_node: None,
    last_updated: SystemTime::now()
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
          log: String::new(),
          status: false
        });
      },
      _ => {
        println!("Error reading file");
      }
    }
  }

  state.table_state.select(Some(0));
  loop {
    if state.last_updated.elapsed().unwrap() > Duration::milliseconds(1000) {
      state.last_updated = SystemTime::now();
      for node in &mut state.nodes {
        node.status = get_ok(node.address.clone(), "/ok".to_string()).await.unwrap();
      }
    }
    let mut rows: Vec<Row> = Vec::new();
    for a in state.nodes.clone() {
      rows.push(node_row(&a));
    }

    terminal.draw(|f| {
      match state.current_page {
        Pages::NodeList => {
          node_list_page(&mut state, rows, f);
        },
        Pages::NodeDetail => {
          node_detail_page(&mut state, f);
        },
        Pages::AddNode => {
          let size = f.size();
          let divs = Layout::default().direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(size);

          f.render_widget(menu(&mut state), divs[0]);
        }
      }
    })?;

    if let Event::Key(key) = event::read()? {
      match key.code {
        event::KeyCode::Char('q') => {
          match state.current_page {
            Pages::NodeDetail => {
              state.current_page = Pages::NodeList;
            },
            _ => {
              break;
            }
          }
        },
        event::KeyCode::Down => {
          let i = state.table_state.selected().unwrap();
          if i < state.nodes.len() - 1 {
            state.table_state.select(Some(i + 1));
          }
        },
        event::KeyCode::Up => {
          let i = state.table_state.selected().unwrap();
          if i > 0 {
            state.table_state.select(Some(i - 1));
          }
        },
        event::KeyCode::Right => {
          match state.current_page {
            Pages::NodeList => {
              state.current_page = Pages::AddNode;
            },
            _ => {}
          }
        },
        event::KeyCode::Left => {
          match state.current_page {
            Pages::AddNode => {
              state.current_page = Pages::NodeList;
            },
            _ => {}
          }
        },
        event::KeyCode::Enter => {
          let selected_idx = state.table_state.selected().unwrap();
          state.nodes[selected_idx].log = get_log(state.nodes[selected_idx].address.clone()).await;
          state.selected_node = Some(state.nodes[selected_idx].clone());
          state.current_page = Pages::NodeDetail;
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

fn node_list_page(mut state: &mut GlobalState, mut rows: Vec<Row>, f: &mut Frame<CrosstermBackend<&mut Stdout>>) {
  let size = f.size();
  let divs = Layout::default().direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
    .split(size);

  f.render_widget(menu(state), divs[0]);


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
    .highlight_style(tui::style::Style::default().add_modifier(tui::style::Modifier::REVERSED))
    .widths(&[Constraint::Percentage(25), Constraint::Percentage(50), Constraint::Percentage(25)]);
  f.render_stateful_widget(table, divs[1], &mut state.table_state);
}

fn menu(mut state: &mut GlobalState) -> Tabs {
  let titles = ["Homepage", "Add Node"].iter().cloned().map(Spans::from).collect();
  Tabs::new(titles)
    .block(Block::default().title("Menu").borders(Borders::ALL))
    .style(Style::default().fg(Color::White))
    .highlight_style(Style::default().fg(Color::Yellow))
    .select(match state.current_page {
      Pages::NodeList => 0,
      Pages::AddNode => 1,
      _ => 0
    })
    .divider(DOT)
}

fn node_detail_page(mut state: &mut GlobalState, f: &mut Frame<CrosstermBackend<&mut Stdout>>){
  let divs = Layout::default().direction(Direction::Vertical)
    .constraints([Constraint::Length(8), Constraint::Length(3)].as_ref())
    .split(f.size());
  let identity_block = Block::default()
    .title("Node Info")
    .borders(Borders::ALL);
  let node_info_text = Paragraph::new(
    format!("Name: {}\nAddress: {}\nLog:\n{}", 
      state.selected_node.as_ref().unwrap().name, 
      state.selected_node.as_ref().unwrap().address,
      state.selected_node.as_ref().unwrap().log))
    .block(identity_block)
    .wrap(Wrap { trim: true });
  f.render_widget(node_info_text, divs[0]);
}

fn node_row(node: &Node) -> Row<'static> {
  return Row::new(vec![
    Span::styled(node.name.clone(), Style::default().add_modifier(Modifier::BOLD)),
    Span::raw(node.address.clone()),
    Span::raw(node.status.to_string()),
  ]);
}

async fn get_log(address: String) -> String {
  let response = get_body(address.clone(), REQUEST_LOG_ROUTE.to_string()).await.unwrap();
  if response == String::new() {
    return "No log".to_string();
  }
  return response;
}

async fn get_ok(address: String, path: String) -> Result<bool, Box<dyn Error>> {
  let client = Client::new();
  let url = format!("{}{}", address, path);
  match client.get(url).send().await {
    Ok(response) => {
      Ok(true)
    },
    Err(err) => {
      Ok(false)
    },
  }
}
async fn get_body(address: String, path: String) -> Result<String, Box<dyn Error>> {
  let client = Client::new();
  let url = format!("{}{}", address, path);
  match client.get(url).send().await {
    Ok(response) => {
      Ok(response.text().await.unwrap())
    },
    Err(err) => {
      Ok(String::new())
    },
  }
}