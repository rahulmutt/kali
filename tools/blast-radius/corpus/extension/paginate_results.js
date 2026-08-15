// Paginate a search result set: clamp the requested page, slice the window,
// and build the page-number strip with ellipses that every result list shows
// at the bottom.

const RESULTS = [];
for (let i = 1; i <= 137; i++) {
  RESULTS.push({ id: 1000 + i, title: "record " + i, score: (i * 37) % 101 });
}

function clamp(value, low, high) {
  if (value < low) return low;
  if (value > high) return high;
  return value;
}

function paginate(items, page = 1, perPage = 20) {
  const pageCount = Math.max(1, Math.ceil(items.length / perPage));
  const current = clamp(Math.floor(page), 1, pageCount);
  const start = (current - 1) * perPage;
  return {
    items: items.slice(start, start + perPage),
    page: current,
    pageCount: pageCount,
    total: items.length,
    hasPrevious: current > 1,
    hasNext: current < pageCount,
  };
}

function pageStrip(current, pageCount, window = 1) {
  const strip = [];
  let lastShown = 0;
  for (let page = 1; page <= pageCount; page++) {
    const edge = page === 1 || page === pageCount;
    const near = Math.abs(page - current) <= window;
    if (!edge && !near) continue;
    if (lastShown !== 0 && page - lastShown > 1) strip.push("...");
    strip.push(page === current ? "[" + page + "]" : String(page));
    lastShown = page;
  }
  return strip.join(" ");
}

function describe(view) {
  const first = (view.page - 1) * view.items.length + 1;
  return "showing " + view.items.length + " of " + view.total + " from #" + first;
}

for (const requested of [1, 4, 7, 99, 0]) {
  const view = paginate(RESULTS, requested);
  console.log(
    "requested " + requested + " -> page " + view.page + "/" + view.pageCount +
      "  " + pageStrip(view.page, view.pageCount),
  );
}

const view = paginate(RESULTS, 4);
console.log(describe(view));
console.log("first row:", view.items[0].id, view.items[0].title);
console.log("last row:", view.items[view.items.length - 1].id);
console.log("previous available:", view.hasPrevious, "next available:", view.hasNext);

const lastPage = paginate(RESULTS, 99);
console.log("last page holds the remainder:", lastPage.items.length === RESULTS.length % 20);
console.log("no page is empty:", paginate([], 1).pageCount === 1);
