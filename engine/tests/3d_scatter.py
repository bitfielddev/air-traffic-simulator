import matplotlib.pyplot as plt

coords = [(0, 0, 0)]
while (c := input()) != "":
    coords.append((float(c.split(" ")[0]), float(c.split(" ")[1]), float(c.split(" ")[2])))

fig = plt.figure()
ax = fig.add_subplot(projection='3d')

ax.scatter([c[0] for c in coords], [c[1] for c in coords], [c[2] for c in coords], marker='o')

plt.show()