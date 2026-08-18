

use std::collections::HashMap;
use std::fs::File;
use cozy_chess::Color;
use engine::Eval;
use plotly::color::NamedColor;
use crate::binpack::{BinPackReader, GameResult};
use plotly::{Plot, Scatter, Layout};
use plotly::common::Mode;

const PLY_SCALING_FACTOR: f64 = 100.0;

// Grid search parameters for k and q tuning
const K_MIN: f64 = 5.0;
const K_MAX: f64 = 1000.0;
const K_STEP: f64 = 1.0;
const Q_MIN: f64 = 0.0;
const Q_MAX: f64 = 5.0;
const Q_STEP: f64 = 0.005;
const GRID_SEARCH_PROGRESS_INTERVAL: usize = 50000;

pub fn compute(binpack_files: &[String], recompute_interval: usize, show_graph: bool) {
    // Create readers for each file
    let mut readers: Vec<_> = binpack_files
        .iter()
        .filter_map(|path| {
            File::open(path)
                .ok()
                .and_then(|f| BinPackReader::new(f).ok())
        })
        .collect();

    if readers.is_empty() {
        eprintln!("No valid binpack files could be opened");
        return;
    }

    println!("Opened {} binpack files", readers.len());

    // Histogram: eval value -> (sum of results, count)
    let mut histogram: HashMap<i32, (f64, u64)> = HashMap::new();
    let mut total_entries = 0u64;
    let mut active_readers = (0..readers.len()).collect::<Vec<_>>();

    // Keep reading until all files exhausted (interleaved)
    while !active_readers.is_empty() {
        let mut i = 0;
        while i < active_readers.len() {
            let reader_idx = active_readers[i];
            
            if let Ok(entry) = readers[reader_idx].get_next_entry() {
                // Convert result from player to move's perspective
                let player_result = match (entry.result, entry.board.side_to_move()) {
                    // Player to move wins
                    (GameResult::WhiteWins, Color::White) => 1.0,
                    (GameResult::BlackWins, Color::Black) => 1.0,
                    // Player to move loses
                    (GameResult::WhiteWins, Color::Black) => 0.0,
                    (GameResult::BlackWins, Color::White) => 0.0,
                    // Draw
                    (GameResult::Draw, _) => 0.5,
                };

                let Eval::CentiPawn(eval_value) = entry.eval else { continue; };
                
                let entry_data = histogram.entry(eval_value).or_insert((0.0, 0));
                entry_data.0 += player_result;
                entry_data.1 += 1;
                
                total_entries += 1;

                // Print progress
                if total_entries % recompute_interval as u64 == 0 {
                    println!("Processed {} entries", total_entries);
                    let (k, q) = tune_k_and_q(&histogram);
                    println!("Optimal k value: {:.2}, q value: {:.6}", k, q);
                    println!("Final MSE: {:.6}", calculate_mse(&histogram, k, q));
                    if show_graph {
                        update_plot(&histogram, k, q);
                    }
                }
                
                i += 1;
            } else {
                // This reader is exhausted, remove it
                active_readers.swap_remove(i);
            }
        }
    }

    println!("Total entries processed: {}", total_entries);
    println!("Unique eval values: {}", histogram.len());

    // Tune k and q using gradient descent
    let (k, q) = tune_k_and_q(&histogram);
    println!("Optimal k value: {:.2}, q value: {:.6}", k, q);
    println!("Final MSE: {:.6}", calculate_mse(&histogram, k, q));
    if show_graph {
        update_plot(&histogram, k, q);
    }
}

fn tune_k_and_q(histogram: &HashMap<i32, (f64, u64)>) -> (f64, f64) {
    let mut best_k = K_MIN;
    let mut best_q = Q_MIN;
    let mut best_mse = f64::INFINITY;
    
    let mut iteration = 0;
    let total_iterations = ((K_MAX - K_MIN) / K_STEP + 1.0) * ((Q_MAX - Q_MIN) / Q_STEP + 1.0);
    
    let mut k = K_MIN;
    while k <= K_MAX {
        let mut q = Q_MIN;
        while q <= Q_MAX {
            let mse = calculate_mse(histogram, k, q);
            
            if mse < best_mse {
                best_mse = mse;
                best_k = k;
                best_q = q;
            }
            
            iteration += 1;
            if iteration % GRID_SEARCH_PROGRESS_INTERVAL == 0 {
                println!("Grid search progress: {:.1}% - Current best: k={:.2}, q={:.4}, MSE={:.8}", 
                         (iteration as f64 / total_iterations) * 100.0, best_k, best_q, best_mse);
            }
            
            q += Q_STEP;
        }
        k += K_STEP;
    }
    
    println!("Grid search complete!");
    println!("Best k: {:.2}, Best q: {:.4}, Best MSE: {:.8}", best_k, best_q, best_mse);
    
    (best_k, best_q)
}

fn calculate_mse(histogram: &HashMap<i32, (f64, u64)>, k: f64, q: f64) -> f64 {
    let mut total_error = 0.0;
    let mut total_weight = 0u64;

    for (eval_value, (sum_result, count)) in histogram.iter() {
        let actual_result = sum_result / *count as f64;
        let predicted = sigmoid(*eval_value as f64, k, q);
        let error = (actual_result - predicted).powi(2);
        
        total_error += error * *count as f64;
        total_weight += count;
    }

    if total_weight == 0 {
        return f64::INFINITY;
    }

    total_error / total_weight as f64
}

fn sigmoid(x: f64, k: f64, q: f64) -> f64 {
    // f(x) = 1 / (1 + e^(-x))
    // g(x) = f(k*x*|k*x|^q)
    let kx = x / k;
    let exponent = kx * kx.abs().powf(q);
    1.0 / (1.0 + (-exponent).exp())
}

fn update_plot(histogram: &HashMap<i32, (f64, u64)>, k: f64, q: f64) {
    let mut plot = Plot::new();
    
    // Extract histogram points
    let mut eval_values: Vec<_> = histogram.keys().copied().collect();
    eval_values.sort();
    
    let xs: Vec<f64> = eval_values.iter().map(|&x| x as f64).collect();
    let ys: Vec<f64> = eval_values
        .iter()
        .map(|&x| {
            let (sum_result, count) = histogram[&x];
            sum_result / count as f64
        })
        .collect();
    
    // Add histogram points
    let scatter = Scatter::new(xs.clone(), ys.clone())
        .mode(Mode::Markers)
        .name("Histogram Points")
        .marker(plotly::common::Marker::new().size(6).color(NamedColor::Blue));
    plot.add_trace(scatter);
    
    // Generate sigmoid curve
    let min_eval = *eval_values.first().unwrap_or(&-3000) as f64;
    let max_eval = *eval_values.last().unwrap_or(&3000) as f64;
    let curve_xs: Vec<f64> = (0..1000)
        .map(|i| min_eval + (max_eval - min_eval) * i as f64 / 999.0)
        .collect();
    let curve_ys: Vec<f64> = curve_xs.iter().map(|&x| sigmoid(x, k, q)).collect();
    
    let curve = Scatter::new(curve_xs, curve_ys)
        .mode(Mode::Lines)
        .name(format!("Sigmoid (k={:.2}, q={:.6})", k, q))
        .line(plotly::common::Line::new().color(NamedColor::Red).width(2.0));
    plot.add_trace(curve);
    
    // Update layout
    let layout = Layout::new()
        .title("Eval Value vs Game Result".into())
        .x_axis(plotly::layout::Axis::new().title("Eval Value".into()))
        .y_axis(plotly::layout::Axis::new().title("Actual Result".into()));
    plot.set_layout(layout);
    
    // Show plot in browser
    plot.show();
}