import matplotlib.pyplot as plt
import numpy as np


x = np.arange(0.01, 30, 0.01)

plt.xscale("log")
plt.yscale("log")
plt.plot(x, np.exp(-x), marker="o", markevery=0.1)
plt.plot(x, 2*np.exp(-x), marker=".", markevery=0.1)

plt.show()
