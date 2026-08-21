import math
import numpy as np
from scipy.optimize import curve_fit


# BATCH_SIZE = 8192
# EPOCHS = 50
# NUM_WORKERS = 24



_FEATURE_TYPES = {
    'SPC': ['SPC', 768],
    'HALF_KP': ['HalfKP', 40960]
}

(FEATURE_SET, NUM_FEATURES) = _FEATURE_TYPES['HALF_KP']


LR = 1e-3
M = 32
K = 32

WEIGHT_SCALE = 64
ACTIVATION_RANGE = 127
BIAS_SCALE = ACTIVATION_RANGE * WEIGHT_SCALE

MIN = -128 / WEIGHT_SCALE
MAX = 127 / WEIGHT_SCALE

MOM_MIN = 14
MOM_TARGET = 58
MOM_MAX = 78

COEFFS_A = [-366.8397619040089, 342.910875925736, -605.8144728281717, 342.2350804516123]
COEFFS_B = [71.65912154840763, -72.77737522323093, 100.03098608150086, -25.240892630322772]


def sigmoid(x):
    if x < -709:
        return 0

    return 1 / (1 + math.exp(-x))


def poly3(x, coeffs):
    """
    Evaluate a cubic polynomial at x with given coefficients.
    coeffs: [c0, c1, c2, c3] for c0 + c1*x + c2*x^2 + c3*x^3
    """
    return ((coeffs[3] * x + coeffs[2]) * x + coeffs[1]) * x + coeffs[0]


def win_rate(x, normalized_mom, coeffs):
    p_a = poly3(normalized_mom, coeffs[:4])
    p_b = poly3(normalized_mom, coeffs[4:])

    z_val = (x - p_a) / p_b
    z_val = np.clip(z_val, -80, 80)

    return 1.0 / (1.0 + np.exp(-z_val))

def score(x, normalized_mom, coeffs=None):
    if coeffs is None:
        coeffs = COEFFS_A + COEFFS_B

    wr = win_rate(x, normalized_mom, coeffs)
    lr = win_rate(-x, normalized_mom, coeffs)
    dr = 1 - wr - lr

    return 1 * wr + 0.5 * dr # + 0 * lr