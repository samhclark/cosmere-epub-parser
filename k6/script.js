import http from 'k6/http';

import { sleep } from 'k6';

export function setup() {
    const res = http.get("https://raw.githubusercontent.com/dwyl/english-words/master/words_dictionary.json")
    const words = Object.keys(res.json());
    return { words: words };
}

export default function (data) {
    http.get('https://csearch-test.fly.dev');
    sleep(1);
    const randomWord = data.words[Math.floor(Math.random() * data.words.length)];
    http.get(`https://csearch-test.fly.dev/search?q=${randomWord}`)
}