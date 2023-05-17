const dropZone = document.querySelector("#drop-zone");
const progressZone = document.querySelector("#progress-zone");
const fileInput = document.querySelector("#file-input");
const progress = document.querySelector("#progress-bar");
const resultZone = document.querySelector("#result-zone");
const copyJsonButton = document.querySelector("#copy-json-button");

function dropHandler(ev) {
  ev.preventDefault();
  const file = ev.dataTransfer.items[0].getAsFile();
  processFile(file);
}

function dragOverHandler(ev) {
  ev.preventDefault();
}

fileInput.addEventListener("change", () => {
  processFile(fileInput.files[0]);
});

function processFile(file) {
  dropZone.classList.add("hidden");
  progressZone.classList.remove("hidden");

  const worker = new Worker("worker.js");
  worker.postMessage(file);
  worker.onmessage = function (ev) {
    const { data } = ev;
    if ("progress" in data) {
      progress.value = data.progress;
    } else if ("result" in data) {
      processResult(data.result);
    } else if ("error" in data) {
      processError(data.error);
    }
  };
}

function processResult(result) {
  progressZone.classList.add("hidden");
  resultZone.classList.remove("hidden");

  copyJsonButton.addEventListener("click", async () => {
    await navigator.clipboard.writeText(
      JSON.stringify(
        result,
        // BigInt can't be serialized natively by JSON. So we turn those values
        // into string.
        (key, value) => (typeof value == "bigint" ? value.toString() : value),
        2
      )
    );
  });

  const treeView = document.querySelector("#tree-view");

  for (element of result) {
    treeView.appendChild(createNode(element));
  }
}

function createNode(element) {
  if ("children" in element) {
    const { id, children, size } = element;
    return createMasterNode(id, children, size);
  } else if ("value" in element) {
    const { id, value, size } = element;
    return createLeaf(id, value, size);
  }
}

function createMasterNode(id, children, size) {
  let masterNode = document.createElement("li");

  let span = document.createElement("span");
  span.classList.add("caret");
  span.textContent = `${id} (size: ${size})`;
  span.addEventListener("click", function () {
    this.parentElement.querySelector(".nested").classList.toggle("active");
    this.classList.toggle("caret-down");
  });
  masterNode.appendChild(span);

  let ul = document.createElement("ul");
  ul.classList.add("nested");
  for (element of children) {
    ul.appendChild(createNode(element));
  }
  masterNode.appendChild(ul);

  return masterNode;
}

function createLeaf(id, value, size) {
  let leaf = document.createElement("li");
  leaf.textContent = `${id}: ${value} (size: ${size})`;
  return leaf;
}

function processError(error) {
  progressZone.classList.add("hidden");
  resultZone.classList.remove("hidden");
  resultZone.textContent = `Error: ${error}`;
}
