import matplotlib.pyplot as plt
import numpy as np
records = []

with open("output.txt", "r") as f:
    record = dict()

    asks = False
    bids = False
    len_ = False
    time = False
    for line in f.readlines():
        if line == "INCOMING\n":
            records.append(record)
            record = dict()
            record["bids"] = []
            record["asks"] = []
            continue
        elif line == "AFTER CLEARING\n":
            pass
        elif line == "BIDS\n":
            bids = True
            len_ = True
            continue
        elif line == "ASKS\n":
            bids = False
            asks = True
            len_ = True
            continue
        elif line == "\n":
            asks = False
        elif line == "===\n":
            time = True
            continue
        
        if asks:
            if len_:
                record["ask_length"] = int(line.rstrip().split(" ")[1])
                len_ = False
                continue
            parse = line.rstrip().split(" \t ")
            parse[0] = float(parse[0][5:-2])
            parse[1] = int(parse[1])
            record["asks"].append(parse)
        if bids:
            if len_:
                record["bid_length"] = int(line.rstrip().split(" ")[1])
                len_ = False
                continue
            parse = line.rstrip().split(" \t ")
            parse[0] = float(parse[0][5:-2])
            parse[1] = int(parse[1])
            record["bids"].append(parse)
        if time:
            record["time"] = float(line.rstrip())
            time = False

            

records = records[1:]
print(len(records))



record = records[1000]
# print(record)

fig,ax = plt.subplots()
for i, record in enumerate(records):
    bids = record["bids"]
    bids = [np.repeat(x[0], x[1]) for x in bids]
    if bids == []:
        bids = np.array([])
    else:
        bids = np.concatenate(bids)

    asks = record["asks"]
    asks = [np.repeat(x[0], x[1]) for x in asks]
    if asks == []:
        asks = np.array([])
    else:
        asks = np.concatenate(asks)

    bins = np.linspace(80, 120, 100)
    
    ax.cla()
    ax.set_ylim(0, 400)
    ax.hist(asks, bins, alpha=0.5, label="asks", density=False)
    ax.hist(bids, bins, alpha=0.5, label="bids", density=False)
    ax.legend(loc="best")
    plt.savefig(f"output/{i:04}.png")
