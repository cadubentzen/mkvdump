import { parse_mkv } from "mkvdump-wasm";

const mkvInput = document.getElementById("mkv-input");
const mkvdumpPre = document.getElementById("mkvdump");

mkvInput.addEventListener("change", async (event) => {
  mkvdumpPre.textContent = "";

  if (event.target.files.length == 0) {
    return;
  }

  const mkvFile = event.target.files[0];
  console.log(mkvFile);

  let mkvContent = await mkvFile.arrayBuffer();
  const mkvView = new Uint8Array(mkvContent);
  const mkvDump = parse_mkv(mkvView);
  console.log(mkvDump);
  mkvdumpPre.textContent = JSON.stringify(mkvDump, null, 2);
});
