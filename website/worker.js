onmessage = async (ev) => {
  for (let i = 0; i <= 100; i++) {
    postMessage({ progress: i });
    await sleep(10);
  }
  //   postMessage({ error: "failed to parse with given buffer size" });
  postMessage({
    result: [
      {
        id: "EBML",
        header_size: 5,
        size: 40,
        children: [
          {
            id: "EBMLVersion",
            header_size: 3,
            size: 4,
            value: 1,
          },
          {
            id: "EBMLReadVersion",
            header_size: 3,
            size: 4,
            value: 1,
          },
        ],
      },
      {
        id: "Segment",
        header_size: 12,
        size: 3859,
        children: [
          {
            id: "SeekHead",
            header_size: 5,
            size: 70,
            children: [
              {
                id: "CRC-32",
                header_size: 2,
                size: 6,
                value: "[c4 b0 cf 83]",
              },
              {
                id: "Seek",
                header_size: 3,
                size: 14,
                children: [
                  {
                    id: "SeekID",
                    header_size: 3,
                    size: 7,
                    value: "Info",
                  },
                  {
                    id: "SeekPosition",
                    header_size: 3,
                    size: 4,
                    value: 229,
                  },
                ],
              },
            ],
          },
        ],
      },
    ],
  });
};

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
