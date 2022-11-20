import { SharedArray } from 'k6/data';
import http from "k6/http";

export const options = {
  discardResponseBodies: true,
  noVUConnectionReuse: true,
  scenarios: {
    open_model: {
      executor: "constant-arrival-rate",
      rate: 512,
      timeUnit: "1s",
      duration: "1m",
      preAllocatedVUs: 200,
    },
  },
};

const data = new SharedArray('some name', function () {
    // All heavy work (opening and processing big files for example) should be done inside here.
    // This way it will happen only once and the result will be shared between all VUs, saving time and memory.
    const f = Object.keys(JSON.parse(open('./wordlist.json')));
    return f; // f must be an array
});

export default function () {
  let randomWord = data[Math.floor(Math.random() * data.length)];
//   http.get(`https://csearch.samhclark.com/search?q=${randomWord}`);
  http.get(`http://127.0.0.1:8080/search?q=${randomWord}`);
}
