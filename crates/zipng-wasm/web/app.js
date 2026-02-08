/**
 * Web application for zipng polyglot generator.
 */

import { initWasm } from "./lib/zipng-loader.js";

const MAX_FILE_SIZE = 60 * 1024; // 60KB

// State
let selectedFiles = [];
let wasmApi = null;
let generatedBlob = null;

// DOM elements
const dropZone = document.getElementById("dropZone");
const fileInput = document.getElementById("fileInput");
const fileListSection = document.getElementById("fileListSection");
const fileList = document.getElementById("fileList");
const clearFilesBtn = document.getElementById("clearFiles");
const generateBtn = document.getElementById("generateBtn");
const loader = document.getElementById("loader");
const previewSection = document.getElementById("previewSection");
const previewImage = document.getElementById("previewImage");
const outputSize = document.getElementById("outputSize");
const downloadBtn = document.getElementById("downloadBtn");
const errorDisplay = document.getElementById("errorDisplay");
const versionSpan = document.getElementById("version");
const modeSelect = document.getElementById("mode");
const fontSelect = document.getElementById("font");
const sortModeSelect = document.getElementById("sortMode");

// Initialize
async function init() {
  try {
    wasmApi = await initWasm();
    versionSpan.textContent = wasmApi.version();
  } catch (err) {
    showError(`Failed to load WASM: ${err.message}`);
    console.error(err);
  }
}

// Event listeners
dropZone.addEventListener("click", () => fileInput.click());
dropZone.addEventListener("dragover", handleDragOver);
dropZone.addEventListener("dragleave", handleDragLeave);
dropZone.addEventListener("drop", handleDrop);
fileInput.addEventListener("change", handleFileSelect);
clearFilesBtn.addEventListener("click", clearFiles);
generateBtn.addEventListener("click", generatePolyglot);
downloadBtn.addEventListener("click", downloadResult);

// Drag and drop handlers
function handleDragOver(e) {
  e.preventDefault();
  dropZone.classList.add("drag-over");
}

function handleDragLeave(e) {
  e.preventDefault();
  dropZone.classList.remove("drag-over");
}

function handleDrop(e) {
  e.preventDefault();
  dropZone.classList.remove("drag-over");

  const files = Array.from(e.dataTransfer.files);
  addFiles(files);
}

function handleFileSelect(e) {
  const files = Array.from(e.target.files);
  addFiles(files);
  fileInput.value = ""; // Reset input
}

// File management
async function addFiles(files) {
  hideError();

  for (const file of files) {
    // Validate size
    if (file.size > MAX_FILE_SIZE) {
      showError(
        `File '${file.name}' exceeds 60KB limit (${formatBytes(file.size)})`
      );
      continue;
    }

    // Read file
    try {
      const content = await readFileAsArrayBuffer(file);
      selectedFiles.push({
        name: file.name,
        size: file.size,
        content: new Uint8Array(content),
      });
    } catch (err) {
      showError(`Failed to read '${file.name}': ${err.message}`);
    }
  }

  updateFileList();
}

function readFileAsArrayBuffer(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result);
    reader.onerror = () => reject(reader.error);
    reader.readAsArrayBuffer(file);
  });
}

function removeFile(index) {
  selectedFiles.splice(index, 1);
  updateFileList();
}

function clearFiles() {
  selectedFiles = [];
  updateFileList();
  hidePreview();
  hideError();
}

function updateFileList() {
  if (selectedFiles.length === 0) {
    fileListSection.classList.add("hidden");
    generateBtn.disabled = true;
    return;
  }

  fileListSection.classList.remove("hidden");
  generateBtn.disabled = false;

  fileList.innerHTML = selectedFiles
    .map(
      (file, index) => `
    <li class="file-item">
      <div class="file-info">
        <div class="file-name">${escapeHtml(file.name)}</div>
        <div class="file-size">${formatBytes(file.size)}</div>
      </div>
      <button class="file-remove" onclick="window.removeFileAt(${index})" title="Remove">
        ×
      </button>
    </li>
  `
    )
    .join("");
}

// Export for inline onclick (simpler than adding event listeners dynamically)
window.removeFileAt = removeFile;

// Generate polyglot
async function generatePolyglot() {
  if (!wasmApi) {
    showError("WASM not loaded yet");
    return;
  }

  if (selectedFiles.length === 0) {
    showError("No files selected");
    return;
  }

  hideError();
  hidePreview();
  generateBtn.disabled = true;
  loader.classList.remove("hidden");

  try {
    // Build options
    const options = {};

    const mode = modeSelect.value;
    if (mode) options.mode = mode;

    const font = fontSelect.value;
    if (font) options.font = font;

    const sortMode = sortModeSelect.value;
    if (sortMode) options.sort_mode = sortMode;

    // Prepare files for WASM
    const files = selectedFiles.map((f) => ({
      path: f.name,
      content: Array.from(f.content),
    }));

    const input = {
      files,
      options,
    };

    // Encode
    const inputJson = JSON.stringify(input);
    const output = wasmApi.encode(inputJson);

    // Create blob
    generatedBlob = new Blob([output], { type: "image/png" });

    // Display preview
    const url = URL.createObjectURL(generatedBlob);
    previewImage.src = url;
    outputSize.textContent = `Output size: ${formatBytes(output.length)}`;
    previewSection.classList.remove("hidden");
  } catch (err) {
    showError(`Encoding failed: ${err.message || err}`);
    console.error(err);
  } finally {
    loader.classList.add("hidden");
    generateBtn.disabled = false;
  }
}

// Download
function downloadResult() {
  if (!generatedBlob) return;

  const url = URL.createObjectURL(generatedBlob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "polyglot.png";
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

// Error handling
function showError(message) {
  errorDisplay.textContent = message;
  errorDisplay.classList.remove("hidden");
}

function hideError() {
  errorDisplay.classList.add("hidden");
}

function hidePreview() {
  previewSection.classList.add("hidden");
  if (previewImage.src) {
    URL.revokeObjectURL(previewImage.src);
    previewImage.src = "";
  }
  generatedBlob = null;
}

// Utilities
function formatBytes(bytes) {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

function escapeHtml(text) {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

// Start
init();
