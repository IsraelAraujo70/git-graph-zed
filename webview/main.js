const graphContainer = document.querySelector("#graph-container");
const commitSummary = document.querySelector("#summary-commits");
const branchSummary = document.querySelector("#summary-branches");
const tagSummary = document.querySelector("#summary-tags");
const truncatedSummary = document.querySelector("#summary-truncated");
const limitInput = document.querySelector("#history-limit");
const refreshButton = document.querySelector("#refresh-button");

const COLORS = ["#7dc2ff", "#ffa17d", "#89f7c5", "#f9c43f", "#c8a6ff"];

function dedupe(values) {
  return [...new Set(values)].sort();
}

function computeLaneIndex(commit, activeLanes) {
  const existingIndex = activeLanes.findIndex((lane) => lane === commit.oid);
  if (existingIndex >= 0) {
    return existingIndex;
  }
  const emptyIndex = activeLanes.findIndex((lane) => lane === null);
  if (emptyIndex >= 0) {
    activeLanes[emptyIndex] = commit.parents[0] ?? null;
    return emptyIndex;
  }
  activeLanes.push(commit.parents[0] ?? null);
  return activeLanes.length - 1;
}

function updateActiveLanes(commit, activeLanes) {
  const index = activeLanes.findIndex((lane) => lane === commit.oid);
  if (index >= 0) {
    activeLanes[index] = commit.parents[0] ?? null;
  }

  if (commit.parents.length > 1) {
    commit.parents.slice(1).forEach((parent) => {
      const emptyIndex = activeLanes.findIndex((lane) => lane === null);
      if (emptyIndex >= 0) {
        activeLanes[emptyIndex] = parent;
      } else {
        activeLanes.push(parent);
      }
    });
  }

  while (activeLanes.length && activeLanes.at(-1) === null) {
    activeLanes.pop();
  }
}

function renderGraph(graph) {
  if (!graph || !graph.commits?.length) {
    graphContainer.innerHTML =
      '<p class="empty-state">No graph data available. Run /git-graph first.</p>';
    commitSummary.textContent = "0";
    branchSummary.textContent = "0";
    tagSummary.textContent = "0";
    truncatedSummary.textContent = "–";
    return;
  }

  const timeline = document.createElement("ol");
  timeline.className = "timeline";
  const activeLanes = [];

  graph.commits.forEach((commit, rowIndex) => {
    const entry = document.createElement("li");
    entry.className = "timeline-entry";
    const laneIndex = computeLaneIndex(commit, activeLanes);
    const graphColumn = document.createElement("div");
    graphColumn.className = "timeline-graph";
    const lane = document.createElement("div");
    lane.className = "lane";
    lane.style.left = `${laneIndex * 26}px`;
    const node = document.createElement("span");
    node.className = "node";
    node.style.background = COLORS[laneIndex % COLORS.length];
    lane.appendChild(node);
    graphColumn.appendChild(lane);

    const content = document.createElement("article");
    content.className = "entry-content";
    const header = document.createElement("header");
    header.className = "entry-header";
    const title = document.createElement("strong");
    title.textContent = commit.summary || "(no message)";
    const meta = document.createElement("span");
    meta.className = "entry-meta";
    meta.textContent = `${commit.author} • ${commit.relative_time}`;
    header.append(title, meta);

    const details = document.createElement("p");
    details.className = "entry-meta";
    details.textContent = `${commit.short_oid} — ${commit.committed_at}`;

    const decorationRow = document.createElement("div");
    decorationRow.className = "decorations";
    if (commit.decorations.head) {
      const badge = document.createElement("span");
      badge.className = "badge head";
      badge.textContent = `HEAD → ${commit.decorations.head}`;
      decorationRow.appendChild(badge);
    }
    commit.decorations.local_branches.forEach((branch) => {
      const badge = document.createElement("span");
      badge.className = "badge branch";
      badge.textContent = branch;
      decorationRow.appendChild(badge);
    });
    commit.decorations.tags.forEach((tag) => {
      const badge = document.createElement("span");
      badge.className = "badge tag";
      badge.textContent = tag;
      decorationRow.appendChild(badge);
    });

    content.append(header, details);
    if (decorationRow.childElementCount) {
      content.append(decorationRow);
    }

    entry.append(graphColumn, content);
    timeline.appendChild(entry);

    updateActiveLanes(commit, activeLanes);
  });

  const branchSet = new Set();
  const tagSet = new Set();
  graph.commits.forEach((commit) => {
    commit.decorations.local_branches.forEach((branch) => branchSet.add(branch));
    commit.decorations.remote_branches.forEach((branch) => branchSet.add(branch));
    commit.decorations.tags.forEach((tag) => tagSet.add(tag));
  });

  commitSummary.textContent = graph.commits.length.toString();
  branchSummary.textContent = branchSet.size.toString();
  tagSummary.textContent = tagSet.size.toString();
  truncatedSummary.textContent = graph.truncated ? "Truncated" : "Complete";

  graphContainer.innerHTML = "";
  graphContainer.appendChild(timeline);
}

function requestGraphUpdate() {
  if (window.parent) {
    window.parent.postMessage(
      {
        type: "git-graph:request",
        limit: Number(limitInput.value) || 400,
      },
      "*",
    );
  }
}

async function bootstrap() {
  if (window.gitGraphData) {
    renderGraph(window.gitGraphData);
    return;
  }
  try {
    const response = await fetch("./sample-data.json");
    if (!response.ok) {
      throw new Error(response.statusText);
    }
    const sample = await response.json();
    renderGraph(sample);
  } catch (error) {
    console.error("Failed to load sample data:", error);
  }
}

window.addEventListener("message", (event) => {
  if (!event.data || typeof event.data !== "object") {
    return;
  }
  if (event.data.type === "git-graph:data") {
    renderGraph(event.data.payload);
  }
});

refreshButton.addEventListener("click", () => {
  requestGraphUpdate();
});

document.addEventListener("dragover", (event) => {
  event.preventDefault();
  event.dataTransfer.dropEffect = "copy";
});

document.addEventListener("drop", async (event) => {
  event.preventDefault();
  const file = event.dataTransfer.files[0];
  if (!file) return;
  const text = await file.text();
  try {
    const payload = JSON.parse(text);
    renderGraph(payload);
  } catch (error) {
    console.error("Failed to parse dropped file", error);
  }
});

bootstrap();
