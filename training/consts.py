import math
import numpy as np
from scipy.optimize import curve_fit


BATCH_SIZE = 8192
EPOCHS = 50

LR = 1e-3
LAMBDA = 1

NUM_WORKERS = 24

_FEATURE_TYPES = {
    'SPC': ['SPC', 768],
    'HALF_KP': ['HalfKP', 40960]
}

(FEATURE_SET, NUM_FEATURES) = _FEATURE_TYPES['HALF_KP']


M = 32
K = 32


WEIGHT_SCALE = 64
ACTIVATION_RANGE = 127
BIAS_SCALE = ACTIVATION_RANGE * WEIGHT_SCALE

MIN = -128 / WEIGHT_SCALE
MAX = 127 / WEIGHT_SCALE

SIGMOID_FACTOR_1 = 286
SIGMOID_FACTOR_2 = 1.4850


def sigmoid(x):
    if x < -709:
        return 0

    return 1 / (1 + math.exp(-x))


def custom_sigmoid(x):
    kx = SIGMOID_FACTOR_1 * x
    exponent = kx * abs(kx) ** SIGMOID_FACTOR_2

    return sigmoid(exponent)