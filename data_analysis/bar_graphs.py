import json

import matplotlib.pyplot as plt

f = open('benchmark.json')

data = json.load(f)
queries = [['tracks_media_some', 500], [
    'albums_tracks_genre_some', 500], ['albums_tracks_genre_all', 200]]


def duration_to_seconds(duration):
    if duration[-2:] == 'ms':
        return(float(duration[:-2]) / 1000)
    elif duration[-1] == 's':
        return(float(duration[:-2]))
    elif duration[-1] == 'm':
        return(float(duration[:-2]) * 60)


for query in queries[0:1]:
    plt.title('title name')
    plt.xlabel('Requests per second')
    plt.ylabel('Mean response latenency (seconds)')
    for tech in ['poggers', 'hasura', 'postgraphile']:
        for i in range(1, 6):
            rps = i * 500
            seconds = duration_to_seconds(
                data['tracks_media_some on ' + tech][str(rps)])
            plt.plot(rps, seconds, label = tech)
    plt.savefig("image.png")
