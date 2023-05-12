import { parse_mkv } from "mkvdump-wasm";
import "@alenaksu/json-viewer";

const mkvInput = document.getElementById("mkv-input");
const mkvdumpViewer = document.getElementById("mkvdump");

mkvInput.addEventListener("change", async (event) => {
  mkvdumpViewer.data = {};

  if (event.target.files.length == 0) {
    return;
  }

  const mkvFile = event.target.files[0];

  let mkvContent = await mkvFile.arrayBuffer();
  const mkvView = new Uint8Array(mkvContent);
  const mkvDump = parse_mkv(mkvView);
  mkvdumpViewer.data = JSON.parse(
    JSON.stringify(
      mkvDump,
      // BigInt can't be serialized natively by JSON. So we turn those values
      // into string.
      (key, value) => (typeof value == "bigint" ? value.toString() : value)
    )
  );
});
