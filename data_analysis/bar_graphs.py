import json

import matplotlib.pyplot as plt

f = open('benchmark.json')

data = json.load(f)
queries = [['tracks_media_some', 500, 6], [
    'albums_tracks_genre_some', 500, 6], ['albums_tracks_genre_all', 200, 5]]


def duration_to_seconds(duration):
    if duration[-2:] == 'ms':
        return(float(duration[:-2]) / 1000)
    elif duration[-1] == 's':
        return(float(duration[:-2]))
    elif duration[-1] == 'm':
        return(float(duration[:-2]) * 60)


for query in queries:
    plt.title(query[0])
    plt.xlabel('Requests/second')
    plt.ylabel('Mean response latenency (seconds)')
    for tech in ['poggers', 'hasura', 'postgraphile']:
        x = []
        y = []
        for i in range(1, query[2]):
            rps = query[1] * i
            x.append(rps)
            y.append(duration_to_seconds(
                data[query[0] + ' on ' + tech][str(rps)]))
            print(query[0] + ' on ' + tech +  '' + str(rps))
            print(duration_to_seconds( data[query[0] + ' on ' + tech][str(rps)]))
        plt.plot(x, y, label=tech)
    plt.legend()
    plt.savefig(query[0])
    plt.clf()

