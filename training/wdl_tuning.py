from typing import DefaultDict
from scipy.optimize import minimize, differential_evolution

import numpy as np
import json
import argparse
import plotly.graph_objects as go
from consts import poly3, score, MOM_MIN, MOM_TARGET, MOM_MAX


called_count = 0

def objective_function(coeffs, data_arrays):
    global called_count
    called_count += 1

    eval_values, normalized_mom_values, expected_scores = data_arrays

    scores = score(eval_values, normalized_mom_values, coeffs)

    result = np.minimum(np.abs(scores - expected_scores), 0.2).mean()

    if called_count % 100 == 0:
        print(f"Call {called_count}: Err={result:.6f}")

    return result

def visualize_fit(eval_values, normalized_mom_values, expected_scores, coeffs, output_file='wdl_fit_3d.html'):
    """
    Create a 3D visualization of data points and fitted model surface.
    """
    # Create figure
    fig = go.Figure()
    
    # Add scatter plot of data points
    fig.add_trace(go.Scatter3d(
        x=eval_values,
        y=normalized_mom_values,
        z=expected_scores,
        mode='markers',
        marker=dict(
            size=4,
            color=expected_scores,
            colorscale='Viridis',
            showscale=True,
            colorbar=dict(title="Expected Score"),
            opacity=0.7
        ),
        name='Data Points',
        text=[f"eval={e:.0f}, mom={m:.2f}, score={s:.3f}" 
              for e, m, s in zip(eval_values, normalized_mom_values, expected_scores)],
        hovertemplate='<b>Data Point</b><br>%{text}<extra></extra>'
    ))
    
    # Create mesh grid for surface
    eval_range = np.linspace(eval_values.min(), eval_values.max(), 30)
    mom_range = np.linspace(normalized_mom_values.min(), normalized_mom_values.max(), 30)
    eval_mesh, mom_mesh = np.meshgrid(eval_range, mom_range)
    
    # Calculate scores on the mesh
    score_mesh = score(eval_mesh.flatten(), mom_mesh.flatten(), coeffs).reshape(eval_mesh.shape)
    
    # Clip to valid range [0, 1]
    score_mesh = np.clip(score_mesh, 0, 1)
    
    # Add surface plot of fitted model
    fig.add_trace(go.Surface(
        x=eval_mesh,
        y=mom_mesh,
        z=score_mesh,
        colorscale='Plasma',
        showscale=False,
        opacity=0.6,
        name='Fitted Model',
        hovertemplate='<b>Fitted Model</b><br>eval=%{x:.0f}<br>mom=%{y:.2f}<br>score=%{z:.3f}<extra></extra>'
    ))
    
    # Update layout
    fig.update_layout(
        title='WDL Model Fit: Data Points vs Fitted Surface',
        scene=dict(
            xaxis_title='Eval Value',
            yaxis_title='Normalized Material/Move Count',
            zaxis_title='Expected Score',
            camera=dict(
                eye=dict(x=1.5, y=1.5, z=1.3)
            )
        ),
        width=1000,
        height=800,
        hovermode='closest'
    )
    
    # Save to HTML
    fig.write_html(output_file)
    print(f"\n3D visualization saved to {output_file}")

def load_data(args):
    with open(args.data, 'r') as f:
        histogram = json.load(f)


    entries = DefaultDict(lambda: np.zeros(3))

    for item in histogram:
        eval_value = item['eval']
        mom_bucket = item['mom_bucket']
        wins = item['wins']
        losses = item['losses']
        draws = item['draws']

        total_games = wins + losses + draws
        if total_games == 0:
            continue

        eval_value = max(args.eval_min, min(args.eval_max, eval_value))
        mom_bucket = max(MOM_MIN, min(MOM_MAX, mom_bucket))

        value = entries[(eval_value, mom_bucket)]
        entries[(eval_value, mom_bucket)] = value + np.array([wins, losses, draws])

    num_entries = len(entries)
    print(f"Loaded {num_entries} entries from histogram.")

    eval_values = np.zeros(num_entries)
    normalized_mom_values = np.zeros(num_entries)
    expected_scores = np.zeros(num_entries)

    for i, ((eval_value, mom_bucket), counts) in enumerate(entries.items()):
        eval_values[i] = eval_value
        normalized_mom_values[i] = (mom_bucket - MOM_MIN) / MOM_TARGET
        wins, losses, draws = counts
        total_games = wins + losses + draws
        expected_scores[i] = (wins + 0.5 * draws) / total_games

    return eval_values, normalized_mom_values, expected_scores

    
if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Tune WDL model parameters')
    parser.add_argument('--data', type=str, required=True,
                        help='Path to histogram JSON data')
    parser.add_argument('--output', type=str, default='wdl_results.json',
                        help='Output JSON file for optimized parameters')
    parser.add_argument('--method', type=str, default='L-BFGS-B',
                        help='Optimization method for scipy.optimize.minimize')
    parser.add_argument('--plot', action='store_true',
                        help='Generate visualization plots')
    parser.add_argument('--eval-min', type=float, default=-20000,
                        help='Clamp eval values below this to this minimum')
    parser.add_argument('--eval-max', type=float, default=20000,
                        help='Clamp eval values above this to this maximum')
    args = parser.parse_args()

    eval_values, normalized_mom_values, expected_scores = load_data(args)

    # Initialize parameters based on data range
    eval_range = eval_values.max() - eval_values.min()
    eval_center = (eval_values.max() + eval_values.min()) / 2.0
    initial_p_b = eval_range / 6.0  # Spread parameter covers about 3 sigma on each side

    
    initial_params = np.array([-185.71, 504.85, -438.58, 474.05, 89.24, -137.02, 73.29, 47.53])
    
    print(f"\nInitial parameters: {initial_params}")
    print(f"Initial MSE: {objective_function(initial_params, (eval_values, normalized_mom_values, expected_scores)):.6f}")
        
    # Set bounds
    bounds = [
        (-1000, 1000),   # c_a0
        (-1000, 1000),   # c_a1
        (-1000, 1000),   # c_a2
        (-1000, 1000),   # c_a3
        (-1000, 1000),   # c_b0
        (-1000, 1000),   # c_b1
        (-1000, 1000),   # c_b2
        (-1000, 1000),      # c_b3 - must be positive
    ]
    
    
    # Optimize
    print(f"\nOptimizing with {args.method}...")
    if args.method == 'differential_evolution':
                result = differential_evolution(
            lambda coeffs: objective_function(coeffs, (eval_values, normalized_mom_values, expected_scores)),
            bounds=bounds,
            workers=1,  # or -1 for parallel
            updating='deferred',
            maxiter=1000,
        )
    else:
        result = minimize(
            objective_function,
            initial_params,
            args=((eval_values, normalized_mom_values, expected_scores),),
            method=args.method,
            # bounds=bounds,
            options={'maxiter': 5000, 'disp': True, 'xtol': 1e-10}
        )
    
    print(f"\nOptimization complete!")
    print(f"Success: {result.success}")
    print(f"Message: {result.message}")
    print(f"Iterations: {result.nit}")
    print(f"Function evaluations: {result.nfev}")
    print(f"Final MSE: {result.fun:.6f}")
    
    # Extract optimized coefficients
    coeffs_a = result.x[:4]
    coeffs_b = result.x[4:8]
    
    print(f"\nCOEFFS_A = {coeffs_a.tolist()}")
    print(f"COEFFS_B = {coeffs_b.tolist()}")
    
    # Format as polynomial expression
    print(f"\np_a(mom) = (({coeffs_a[0]:.2f} * mom / {MOM_TARGET} + {coeffs_a[1]:.2f}) * mom / {MOM_TARGET} + {coeffs_a[2]:.2f}) * mom / {MOM_TARGET} + {coeffs_a[3]:.2f}")
    print(f"p_b(mom) = (({coeffs_b[0]:.2f} * mom / {MOM_TARGET} + {coeffs_b[1]:.2f}) * mom / {MOM_TARGET} + {coeffs_b[2]:.2f}) * mom / {MOM_TARGET} + {coeffs_b[3]:.2f}")
    
    # Create 3D visualization
    visualize_fit(eval_values, normalized_mom_values, expected_scores, result.x, output_file='wdl_fit_3d.html')
        
