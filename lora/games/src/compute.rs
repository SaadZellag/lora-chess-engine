

use std::collections::HashMap;
use std::fs::File;
use cozy_chess::Color;
use engine::Eval;
use plotly::color::NamedColor;
use crate::binpack::{BinPackReader, GameResult};
use plotly::{Plot, Scatter, Layout};
use plotly::common::Mode;

const PLY_SCALING_FACTOR: f64 = 100.0;

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
                    let k = tune_k(&histogram);
                    println!("Optimal k value: {:.2}", k);
                    println!("Final MSE: {:.6}", calculate_mse(&histogram, k));
                    if show_graph {
                        update_plot(&histogram, k);
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

    // Tune k using ternary search
    let k = tune_k(&histogram);
    println!("Optimal k value: {:.2}", k);
    println!("Final MSE: {:.6}", calculate_mse(&histogram, k));
    if show_graph {
        update_plot(&histogram, k);
    }
}

fn tune_k(histogram: &HashMap<i32, (f64, u64)>) -> f64 {
    let mut low = 1.0;
    let mut high = 10000.0;
    let tolerance = 0.01;

    while high - low > tolerance {
        let mid1 = low + (high - low) / 3.0;
        let mid2 = high - (high - low) / 3.0;
        
        let mse1 = calculate_mse(histogram, mid1);
        let mse2 = calculate_mse(histogram, mid2);
        
        if mse1 > mse2 {
            low = mid1;
        } else {
            high = mid2;
        }
    }

    (low + high) / 2.0
}

fn calculate_mse(histogram: &HashMap<i32, (f64, u64)>, k: f64) -> f64 {
    let mut total_error = 0.0;
    let mut total_weight = 0u64;

    for (eval_value, (sum_result, count)) in histogram.iter() {
        let actual_result = sum_result / *count as f64;
        let predicted = sigmoid(*eval_value as f64 / k);
        let error = (actual_result - predicted).powi(2);
        
        total_error += error * *count as f64;
        total_weight += count;
    }

    if total_weight == 0 {
        return f64::INFINITY;
    }

    total_error / total_weight as f64
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

fn update_plot(histogram: &HashMap<i32, (f64, u64)>, k: f64) {
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
    let curve_ys: Vec<f64> = curve_xs.iter().map(|&x| sigmoid(x / k)).collect();
    
    let curve = Scatter::new(curve_xs, curve_ys)
        .mode(Mode::Lines)
        .name(format!("Sigmoid (k={:.2})", k))
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