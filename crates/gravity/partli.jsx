import React, { useState, useEffect, useRef, useCallback } from 'react';

// Editable value component - click to edit, blur/enter to commit
const EditableValue = ({ fieldName, value, displayValue, onCommit, editingField, setEditingField, editingValue, setEditingValue, style }) => {
  const isEditing = editingField === fieldName;
  
  const handleClick = () => {
    setEditingField(fieldName);
    setEditingValue(String(value));
  };
  
  const handleBlur = () => {
    const num = parseFloat(editingValue);
    if (!isNaN(num)) {
      onCommit(num);
    }
    setEditingField(null);
  };
  
  const handleKeyDown = (e) => {
    if (e.key === 'Enter') {
      e.target.blur();
    } else if (e.key === 'Escape') {
      setEditingField(null);
    }
  };
  
  if (isEditing) {
    return (
      <input
        type="text"
        value={editingValue}
        onChange={e => setEditingValue(e.target.value)}
        onBlur={handleBlur}
        onKeyDown={handleKeyDown}
        autoFocus
        style={{
          width: '60px',
          fontFamily: 'monospace',
          fontSize: 'inherit',
          padding: '1px 4px',
          border: '1px solid #8cf',
          borderRadius: '2px',
          backgroundColor: '#2a2a4e',
          color: '#fff',
          ...style
        }}
      />
    );
  }
  
  return (
    <span
      onClick={handleClick}
      style={{
        cursor: 'pointer',
        borderBottom: '1px dashed #666',
        ...style
      }}
    >
      {displayValue !== undefined ? displayValue : value}
    </span>
  );
};

export default function GameOfLife() {
  // Grid parameters
  const [W, setW] = useState(192);
  const [H, setH] = useState(120);
  const [SX, setSX] = useState(96);
  const [SY, setSY] = useState(0);
  const [wrapEnabled, setWrapEnabled] = useState(true);
  
  // Population limits
  const [PIPercent, setPIPercent] = useState(null);
  const [PIAbsolute, setPIAbsolute] = useState(124);
  const [PAPercent, setPAPercent] = useState(null);
  const [PAAbsolute, setPAAbsolute] = useState(132);
  
  // Rate limits
  const [RBPercent, setRBPercent] = useState(null);
  const [RBAbsolute, setRBAbsolute] = useState(4);
  const [RDPercent, setRDPercent] = useState(null);
  const [RDAbsolute, setRDAbsolute] = useState(4);
  const [RCPercent, setRCPercent] = useState(null);
  const [RCAbsolute, setRCAbsolute] = useState(4);
  
  // Initial density
  const [initialDensity, setInitialDensity] = useState(2/256 * 100);
  
  // Initial velocity ranges (default to half speed limit)
  const [initVelMinX, setInitVelMinX] = useState(-1.125);
  const [initVelMaxX, setInitVelMaxX] = useState(1.125);
  const [initVelMinY, setInitVelMinY] = useState(-1.125);
  const [initVelMaxY, setInitVelMaxY] = useState(1.125);
  
  // Wind resistance (drag proportional to v²)
  const [dragX, setDragX] = useState(0);
  const [dragY, setDragY] = useState(0);
  
  // Particle physics settings
  const [G, setG] = useState(0.5);
  const [GS, setGS] = useState(2.5);
  const [SL, setSL] = useState(2.25);
  const [UD, setUD] = useState(false);
  const [QT, setQT] = useState(0.5);
  
  // Cosmetic settings
  const [hue, setHue] = useState(165);
  const [rotation, setRotation] = useState(0);
  const [FD, setFD] = useState(75);
  const [FP, setFP] = useState(6);
  const [FF, setFF] = useState(127);
  
  // Simulation control
  const [fps, setFps] = useState(24);
  const [running, setRunning] = useState(true);
  const [tick, setTick] = useState(0);
  const [countdown, setCountdown] = useState(256);
  const [renderTrigger, setRenderTrigger] = useState(0);
  const [actualFps, setActualFps] = useState(0);
  const [overlayOpen, setOverlayOpen] = useState(false);
  const [canvasHover, setCanvasHover] = useState(false);
  const [editingField, setEditingField] = useState(null);
  const [editingValue, setEditingValue] = useState('');
  
  // Viewport sizing
  const CONTENT_PADDING = 8;
  const MAX_TARGET_SIZE = 768;
  const [viewportWidth, setViewportWidth] = useState(typeof window !== 'undefined' ? window.innerWidth : 800);
  const [viewportHeight, setViewportHeight] = useState(typeof window !== 'undefined' ? window.innerHeight : 600);
  
  // Common props for EditableValue
  const evProps = { editingField, setEditingField, editingValue, setEditingValue };
  
  // Refs for display
  const maxTickWidthRef = useRef(1);
  const maxPopWidthRef = useRef(1);
  const maxPauseWidthRef = useRef(1);
  const maxFpsWidthRef = useRef(1);
  const maxAspectWidthRef = useRef(3);
  
  // Grid state refs
  const aliveRef = useRef(null);
  const posXRef = useRef(null);
  const posYRef = useRef(null);
  const velXRef = useRef(null);
  const velYRef = useRef(null);
  const movedThisTickRef = useRef(null);
  // Color state: store as hue, saturation, lightness for each cell
  const colorHRef = useRef(null);
  const colorSRef = useRef(null);
  const colorLRef = useRef(null);
  
  const canvasRef = useRef(null);
  const overlayCanvasRef = useRef(null);
  const [tileUrl, setTileUrl] = useState(null);
  const animationRef = useRef(null);
  const countdownRef = useRef(null);
  const frameTimesRef = useRef([]);
  
  // Debounce refs
  const wasRunningBeforeChangeRef = useRef(true);
  const settingsChangeTimeoutRef = useRef(null);
  const autoResumeTimeoutRef = useRef(null);
  const urlUpdateTimeoutRef = useRef(null);
  const manualPauseResumeRef = useRef(false);
  const isFirstChangeInSequenceRef = useRef(true);
  
  // Settings ref for render/step
  const settingsRef = useRef({
    W: 192, H: 120, SX: 96, SY: 0, wrapEnabled: true,
    G: 0.5, GS: 2.5, SL: 2.25, UD: false, QT: 0.5,
    hue: 165, FD: 75, FP: 6, FF: 127,
    initVelMinX: -1.125, initVelMaxX: 1.125, initVelMinY: -1.125, initVelMaxY: 1.125,
    dragX: 0, dragY: 0
  });
  
  useEffect(() => {
    settingsRef.current = { W, H, SX, SY, wrapEnabled, G, GS, SL, UD, QT, hue, FD, FP, FF, initVelMinX, initVelMaxX, initVelMinY, initVelMaxY, dragX, dragY };
  }, [W, H, SX, SY, wrapEnabled, G, GS, SL, UD, QT, hue, FD, FP, FF, initVelMinX, initVelMaxX, initVelMinY, initVelMaxY, dragX, dragY]);
  
  const calculateDefaultStagger = (w, h) => {
    if (w === h) return { sx: 0, sy: 0 };
    if (w % 2 !== 0 && h % 2 !== 0) return { sx: 0, sy: 0 };
    if (w > h && w % 2 === 0) return { sx: Math.floor(w / 2), sy: 0 };
    if (h > w && h % 2 === 0) return { sx: 0, sy: Math.floor(h / 2) };
    if (w % 2 === 0) return { sx: Math.floor(w / 2), sy: 0 };
    return { sx: 0, sy: Math.floor(h / 2) };
  };
  
  const gcd = (a, b) => b === 0 ? a : gcd(b, a % b);
  const getAspectRatio = () => { const g = gcd(W, H); return `${W / g}:${H / g}`; };
  
  // OKLCH to RGB conversion (perceptual color space)
  const oklchToRgb = (l, c, h) => {
    // Convert OKLCH to OKLab
    const hRad = h * Math.PI / 180;
    const a = c * Math.cos(hRad);
    const b = c * Math.sin(hRad);
    
    // OKLab to linear RGB
    const l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    const m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    const s_ = l - 0.0894841775 * a - 1.2914855480 * b;
    
    const l3 = l_ * l_ * l_;
    const m3 = m_ * m_ * m_;
    const s3 = s_ * s_ * s_;
    
    let r = 4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3;
    let g = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3;
    let bl = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3;
    
    // Gamma correction
    const gamma = x => x <= 0 ? 0 : (x >= 1 ? 1 : (x < 0.0031308 ? 12.92 * x : 1.055 * Math.pow(x, 1/2.4) - 0.055));
    
    return [
      Math.round(Math.max(0, Math.min(255, gamma(r) * 255))),
      Math.round(Math.max(0, Math.min(255, gamma(g) * 255))),
      Math.round(Math.max(0, Math.min(255, gamma(bl) * 255)))
    ];
  };
  
  const clearGrid = useCallback((w, h) => {
    const totalCells = w * h;
    aliveRef.current = new Uint8Array(totalCells);
    posXRef.current = new Float32Array(totalCells);
    posYRef.current = new Float32Array(totalCells);
    velXRef.current = new Float32Array(totalCells);
    velYRef.current = new Float32Array(totalCells);
    movedThisTickRef.current = new Uint8Array(totalCells);
    colorHRef.current = new Float32Array(totalCells);
    colorSRef.current = new Float32Array(totalCells);
    colorLRef.current = new Float32Array(totalCells);
  }, []);
  
  const initGridWithParams = useCallback((w, h, density, velMinX = -1.125, velMaxX = 1.125, velMinY = -1.125, velMaxY = 1.125) => {
    const totalCells = w * h;
    const alive = new Uint8Array(totalCells);
    const posX = new Float32Array(totalCells);
    const posY = new Float32Array(totalCells);
    const velX = new Float32Array(totalCells);
    const velY = new Float32Array(totalCells);
    const movedThisTick = new Uint8Array(totalCells);
    const colorH = new Float32Array(totalCells);
    const colorS = new Float32Array(totalCells);
    const colorL = new Float32Array(totalCells);
    
    for (let i = 0; i < totalCells; i++) {
      const x = i % w;
      const y = Math.floor(i / w);
      if (Math.random() * 100 < density) {
        alive[i] = 1;
        posX[i] = x + 0.5;
        posY[i] = y + 0.5;
        velX[i] = velMinX + Math.random() * (velMaxX - velMinX);
        velY[i] = velMinY + Math.random() * (velMaxY - velMinY);
        colorH[i] = 0;
        colorS[i] = 0;
        colorL[i] = 0.7;
      }
    }
    
    aliveRef.current = alive;
    posXRef.current = posX;
    posYRef.current = posY;
    velXRef.current = velX;
    velYRef.current = velY;
    movedThisTickRef.current = movedThisTick;
    colorHRef.current = colorH;
    colorSRef.current = colorS;
    colorLRef.current = colorL;
  }, []);
  
  // Wrap a coordinate with stagger
  const wrapCoord = useCallback((x, y, w, h, sx, sy) => {
    let nx = x, ny = y;
    while (nx < 0) { nx += w; ny += sy; }
    while (nx >= w) { nx -= w; ny -= sy; }
    while (ny < 0) { ny += h; nx += sx; }
    while (ny >= h) { ny -= h; nx -= sx; }
    nx = ((nx % w) + w) % w;
    ny = ((ny % h) + h) % h;
    return [nx, ny];
  }, []);
  
  // Get neighbor cell coord (integer)
  const getNeighborCoord = useCallback((x, y, dx, dy, w, h, sx, sy, wrap) => {
    let nx = x + dx;
    let ny = y + dy;
    if (!wrap) {
      if (nx < 0 || nx >= w || ny < 0 || ny >= h) return null;
      return [nx, ny];
    }
    return wrapCoord(nx, ny, w, h, sx, sy);
  }, [wrapCoord]);
  
  // Get wrapped copies of a position for gravity calculation
  const getWrappedCopies = useCallback((px, py, w, h, sx, sy) => {
    const copies = [[px, py]]; // self in main grid
    
    // Find closest copy above (y >= h)
    let aboveX = px, aboveY = py + h;
    aboveX -= sx;
    copies.push([aboveX, aboveY]);
    
    // Find closest copy below (y < 0)
    let belowX = px, belowY = py - h;
    belowX += sx;
    copies.push([belowX, belowY]);
    
    // Find closest copy right (x >= w)
    let rightX = px + w, rightY = py;
    rightY -= sy;
    copies.push([rightX, rightY]);
    
    // Find closest copy left (x < 0)
    let leftX = px - w, leftY = py;
    leftY += sy;
    copies.push([leftX, leftY]);
    
    return copies;
  }, []);
  
  const render = useCallback(() => {
    const canvas = canvasRef.current;
    const alive = aliveRef.current;
    const colorH = colorHRef.current;
    const colorS = colorSRef.current;
    const colorL = colorLRef.current;
    const { W: w, H: h, SX: sx, SY: sy, wrapEnabled: wrap, hue: baseHue } = settingsRef.current;
    
    if (!canvas || !alive) return;
    
    const ctx = canvas.getContext('2d');
    
    // If not wrapping, render core at 2x filling entire canvas
    // If wrapping, render with surrounding cells
    const canvasW = wrap ? w * 2 : w;
    const canvasH = wrap ? h * 2 : h;
    
    if (canvas.width !== canvasW || canvas.height !== canvasH) {
      canvas.width = canvasW;
      canvas.height = canvasH;
    }
    
    const imageData = ctx.createImageData(canvasW, canvasH);
    const data = imageData.data;
    
    // Dead cell base color (will have baseHue applied)
    const deadL = 0;
    const deadC = 0;
    
    const mapWrapped = (x, y) => {
      if (!wrap) return null;
      let nx = x, ny = y;
      while (nx < 0) { nx += w; ny += sy; }
      while (nx >= w) { nx -= w; ny -= sy; }
      while (ny < 0) { ny += h; nx += sx; }
      while (ny >= h) { ny -= h; nx -= sx; }
      nx = ((nx % w) + w) % w;
      ny = ((ny % h) + h) % h;
      return [nx, ny];
    };
    
    if (wrap) {
      // Render with surrounding wrapped cells
      const offsetX = Math.floor(w / 2);
      const offsetY = Math.floor(h / 2);
      
      for (let cy = 0; cy < canvasH; cy++) {
        for (let cx = 0; cx < canvasW; cx++) {
          const gx = cx - offsetX;
          const gy = cy - offsetY;
          
          let cellH = baseHue, cellC = deadC, cellL = deadL;
          
          let srcIdx = -1;
          let srcX = -1, srcY = -1;
          
          const isCenter = gx >= 0 && gx < w && gy >= 0 && gy < h;
          if (isCenter) {
            srcX = gx;
            srcY = gy;
            srcIdx = gy * w + gx;
          } else {
            const mapped = mapWrapped(gx, gy);
            if (mapped) {
              srcX = mapped[0];
              srcY = mapped[1];
              srcIdx = mapped[1] * w + mapped[0];
            }
          }
          
          if (srcIdx >= 0) {
            if (alive[srcIdx]) {
              // Apply base hue rotation to particle hue
              cellH = (colorH[srcIdx] + baseHue) % 360;
              cellC = colorS[srcIdx];
              cellL = colorL[srcIdx];
            } else if (colorL[srcIdx] > 0) {
              // Fading dead cell - apply base hue rotation
              cellH = (colorH[srcIdx] + baseHue) % 360;
              cellC = colorS[srcIdx] * 0.5;
              cellL = colorL[srcIdx];
            }
          }
          
          // Convert to RGB
          let [r, g, b] = oklchToRgb(cellL, cellC, cellH);
          
          // Lighten inside edges of tiles (12.5% towards white)
          // A pixel is on inside edge if its source coord is at edge of core grid
          if (srcIdx >= 0) {
            const isInsideEdge = srcX === 0 || srcX === w - 1 || srcY === 0 || srcY === h - 1;
            if (isInsideEdge) {
              r = r + (255 - r) * 0.125;
              g = g + (255 - g) * 0.125;
              b = b + (255 - b) * 0.125;
            }
          }
          
          const dstIdx = ((canvasH - 1 - cy) * canvasW + cx) * 4;
          data[dstIdx] = Math.round(r);
          data[dstIdx + 1] = Math.round(g);
          data[dstIdx + 2] = Math.round(b);
          data[dstIdx + 3] = 255;
        }
      }
    } else {
      // No wrapping: render core grid only, filling entire canvas
      for (let cy = 0; cy < canvasH; cy++) {
        for (let cx = 0; cx < canvasW; cx++) {
          const srcIdx = cy * w + cx;
          
          let cellH = baseHue, cellC = deadC, cellL = deadL;
          
          if (alive[srcIdx]) {
            cellH = (colorH[srcIdx] + baseHue) % 360;
            cellC = colorS[srcIdx];
            cellL = colorL[srcIdx];
          } else if (colorL[srcIdx] > 0) {
            cellH = (colorH[srcIdx] + baseHue) % 360;
            cellC = colorS[srcIdx] * 0.5;
            cellL = colorL[srcIdx];
          }
          
          const [r, g, b] = oklchToRgb(cellL, cellC, cellH);
          
          const dstIdx = ((canvasH - 1 - cy) * canvasW + cx) * 4;
          data[dstIdx] = Math.round(r);
          data[dstIdx + 1] = Math.round(g);
          data[dstIdx + 2] = Math.round(b);
          data[dstIdx + 3] = 255;
        }
      }
    }
    
    ctx.putImageData(imageData, 0, 0);
  }, [wrapCoord]);
  
  const recordFrameTime = useCallback(() => {
    const now = performance.now();
    frameTimesRef.current.push(now);
    if (frameTimesRef.current.length > 33) frameTimesRef.current.shift();
    if (frameTimesRef.current.length >= 2) {
      const times = frameTimesRef.current;
      const elapsed = times[times.length - 1] - times[0];
      const frames = times.length - 1;
      setActualFps(Math.round((frames / elapsed) * 1000));
    }
  }, []);
  
  const step = useCallback(() => {
    const alive = aliveRef.current;
    const posX = posXRef.current;
    const posY = posYRef.current;
    const velX = velXRef.current;
    const velY = velYRef.current;
    const movedThisTick = movedThisTickRef.current;
    const colorH = colorHRef.current;
    const colorS = colorSRef.current;
    const colorL = colorLRef.current;
    
    const { W: w, H: h, SX: sx, SY: sy, wrapEnabled: wrap, G: grav, GS: gs, SL: sl, UD: ud, FD: fd, FP: fp, FF: ff, dragX: dx, dragY: dy } = settingsRef.current;
    
    if (!alive) return;
    
    const totalCells = w * h;
    const gs2 = gs * gs;
    
    // Reset moved flags
    for (let i = 0; i < totalCells; i++) movedThisTick[i] = 0;
    
    // Create shuffled global cell order
    const cellOrder = [];
    for (let i = 0; i < totalCells; i++) cellOrder.push(i);
    for (let i = cellOrder.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [cellOrder[i], cellOrder[j]] = [cellOrder[j], cellOrder[i]];
    }
    
    // Helper: count live neighbors at current state
    const countLiveNeighbors = (idx) => {
      const x = idx % w;
      const y = Math.floor(idx / w);
      let count = 0;
      for (let dy = -1; dy <= 1; dy++) {
        for (let dx = -1; dx <= 1; dx++) {
          if (dx === 0 && dy === 0) continue;
          const coord = getNeighborCoord(x, y, dx, dy, w, h, sx, sy, wrap);
          if (coord && alive[coord[1] * w + coord[0]]) count++;
        }
      }
      return count;
    };
    
    // Helper: get live neighbor indices
    const getLiveNeighbors = (idx) => {
      const x = idx % w;
      const y = Math.floor(idx / w);
      const neighbors = [];
      for (let dy = -1; dy <= 1; dy++) {
        for (let dx = -1; dx <= 1; dx++) {
          if (dx === 0 && dy === 0) continue;
          const coord = getNeighborCoord(x, y, dx, dy, w, h, sx, sy, wrap);
          if (coord) {
            const nIdx = coord[1] * w + coord[0];
            if (alive[nIdx]) neighbors.push(nIdx);
          }
        }
      }
      return neighbors;
    };
    
    // ========== CONWAY PHASE ==========
    // Determine births and deaths
    const birthCandidates = [];
    const deathCandidates = [];
    
    for (const idx of cellOrder) {
      const neighbors = countLiveNeighbors(idx);
      if (alive[idx]) {
        // Death check: with UD=false, cells with 0 neighbors don't die
        if (neighbors < 2 || neighbors > 3) {
          if (ud || neighbors > 0) {
            deathCandidates.push(idx);
          }
        }
      } else {
        if (neighbors === 3) {
          birthCandidates.push(idx);
        }
      }
    }
    
    // Get effective value helper
    const getEff = (pct, abs, total) => {
      if (abs !== null) return abs;
      if (pct !== null) return Math.floor(total * pct / 100);
      return total;
    };
    
    let currentPop = 0;
    for (let i = 0; i < totalCells; i++) if (alive[i]) currentPop++;
    
    // Apply rate limits (already in shuffled order from cellOrder filtering)
    const births = [...birthCandidates];
    const deaths = [...deathCandidates];
    
    const maxBirths = getEff(RBPercent, RBAbsolute, currentPop);
    while (births.length > maxBirths) births.pop();
    
    const maxDeaths = getEff(RDPercent, RDAbsolute, currentPop);
    while (deaths.length > maxDeaths) deaths.pop();
    
    const maxChanges = getEff(RCPercent, RCAbsolute, currentPop);
    while (births.length + deaths.length > maxChanges) {
      if (Math.random() < 0.5 && births.length > 0) births.pop();
      else if (deaths.length > 0) deaths.pop();
      else if (births.length > 0) births.pop();
    }
    
    const minPop = getEff(PIPercent, PIAbsolute, totalCells);
    while (deaths.length > 0 && currentPop - deaths.length < minPop) deaths.pop();
    
    const maxPop = getEff(PAPercent, PAAbsolute, totalCells);
    while (births.length > 0 && currentPop + births.length > maxPop) births.pop();
    
    // Merge births and deaths in global order
    const birthSet = new Set(births);
    const deathSet = new Set(deaths);
    const changes = [];
    for (const idx of cellOrder) {
      if (birthSet.has(idx)) changes.push({ type: 'birth', idx });
      else if (deathSet.has(idx)) changes.push({ type: 'death', idx });
    }
    
    // Apply births and deaths with momentum transfer
    for (const change of changes) {
      const idx = change.idx;
      const x = idx % w;
      const y = Math.floor(idx / w);
      
      if (change.type === 'birth') {
        const neighbors = getLiveNeighbors(idx);
        const count = neighbors.length;
        
        if (count > 0) {
          // Calculate born position: average of neighbor positions, clamped to cell
          let avgX = 0, avgY = 0;
          for (const nIdx of neighbors) {
            avgX += posX[nIdx];
            avgY += posY[nIdx];
          }
          avgX /= count;
          avgY /= count;
          
          // Clamp to cell bounds
          posX[idx] = Math.max(x, Math.min(x + 0.9999, avgX));
          posY[idx] = Math.max(y, Math.min(y + 0.9999, avgY));
          
          // Momentum transfer: each neighbor loses 1/(count+1) of velocity
          let sumVX = 0, sumVY = 0;
          const fraction = 1 / (count + 1);
          for (const nIdx of neighbors) {
            const takeX = velX[nIdx] * fraction;
            const takeY = velY[nIdx] * fraction;
            velX[nIdx] -= takeX;
            velY[nIdx] -= takeY;
            sumVX += takeX;
            sumVY += takeY;
          }
          velX[idx] = sumVX;
          velY[idx] = sumVY;
        } else {
          posX[idx] = x + 0.5;
          posY[idx] = y + 0.5;
          velX[idx] = 0;
          velY[idx] = 0;
        }
        
        alive[idx] = 1;
        // Initial color: will be set based on velocity later
        colorL[idx] = 0.7;
        colorS[idx] = 0;
        colorH[idx] = 0;
        
      } else if (change.type === 'death') {
        const neighbors = getLiveNeighbors(idx);
        const count = neighbors.length;
        
        // With UD=false, protect cells that now have 0 neighbors
        if (!ud && count === 0) {
          continue; // Skip this death
        }
        
        // Distribute velocity to neighbors
        if (count > 0) {
          const shareX = velX[idx] / count;
          const shareY = velY[idx] / count;
          for (const nIdx of neighbors) {
            velX[nIdx] += shareX;
            velY[nIdx] += shareY;
          }
        }
        
        alive[idx] = 0;
        // Apply death fade
        const deathFadeTarget = 1.0 - (fd / 100);
        const normalFade = colorL[idx] * (1 - 1/fp) - 1/ff;
        colorL[idx] = Math.min(deathFadeTarget, Math.max(normalFade, 0)) * colorL[idx];
      }
    }
    
    // ========== GRAVITY PHASE ==========
    // Build list of live particles with positions
    const liveParticles = [];
    for (let i = 0; i < totalCells; i++) {
      if (alive[i]) liveParticles.push(i);
    }
    
    const forceX = new Float32Array(totalCells);
    const forceY = new Float32Array(totalCells);
    
    const qt = settingsRef.current.QT;
    
    if (qt === 0 || liveParticles.length < 8) {
      // ===== EXACT O(n²) MODE =====
      for (let i = 0; i < liveParticles.length; i++) {
        const idxA = liveParticles[i];
        const pxA = posX[idxA];
        const pyA = posY[idxA];
        
        const copiesA = wrap ? getWrappedCopies(pxA, pyA, w, h, sx, sy) : [[pxA, pyA]];
        
        for (let j = i + 1; j < liveParticles.length; j++) {
          const idxB = liveParticles[j];
          const pxB = posX[idxB];
          const pyB = posY[idxB];
          
          const copiesB = wrap ? getWrappedCopies(pxB, pyB, w, h, sx, sy) : [[pxB, pyB]];
          
          for (const [bx, by] of copiesB) {
            const dx = bx - pxA;
            const dy = by - pyA;
            const dist2 = dx * dx + dy * dy;
            const force = grav / (dist2 + gs2);
            const dist = Math.sqrt(dist2 + gs2);
            const fx = force * dx / dist;
            const fy = force * dy / dist;
            forceX[idxA] += fx;
            forceY[idxA] += fy;
          }
          
          for (const [ax, ay] of copiesA) {
            const dx = ax - pxB;
            const dy = ay - pyB;
            const dist2 = dx * dx + dy * dy;
            const force = grav / (dist2 + gs2);
            const dist = Math.sqrt(dist2 + gs2);
            const fx = force * dx / dist;
            const fy = force * dy / dist;
            forceX[idxB] += fx;
            forceY[idxB] += fy;
          }
        }
        
        if (wrap) {
          for (let c = 1; c < copiesA.length; c++) {
            const [cx, cy] = copiesA[c];
            const dx = cx - pxA;
            const dy = cy - pyA;
            const dist2 = dx * dx + dy * dy;
            const force = grav / (dist2 + gs2);
            const dist = Math.sqrt(dist2 + gs2);
            forceX[idxA] += force * dx / dist;
            forceY[idxA] += force * dy / dist;
          }
        }
      }
    } else {
      // ===== BARNES-HUT QUADTREE MODE =====
      
      // Collect all particles including ghost copies for wrapping
      const allParticles = [];
      for (const idx of liveParticles) {
        const px = posX[idx];
        const py = posY[idx];
        allParticles.push({ idx, x: px, y: py, isGhost: false });
        
        if (wrap) {
          // Add ghost copies near edges
          const copies = getWrappedCopies(px, py, w, h, sx, sy);
          for (let c = 1; c < copies.length; c++) {
            allParticles.push({ idx, x: copies[c][0], y: copies[c][1], isGhost: true });
          }
        }
      }
      
      // Find bounds
      let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
      for (const p of allParticles) {
        if (p.x < minX) minX = p.x;
        if (p.x > maxX) maxX = p.x;
        if (p.y < minY) minY = p.y;
        if (p.y > maxY) maxY = p.y;
      }
      
      // Pad slightly to avoid edge cases
      const pad = 1;
      minX -= pad; minY -= pad; maxX += pad; maxY += pad;
      const size = Math.max(maxX - minX, maxY - minY);
      
      // Quadtree node structure
      const createNode = (x, y, size) => ({
        x, y, size,
        mass: 0,
        comX: 0, comY: 0,
        particles: [],
        children: null // [NW, NE, SW, SE] when subdivided
      });
      
      const root = createNode(minX, minY, size);
      
      // Insert particle into tree
      const insert = (node, p, depth = 0) => {
        if (depth > 20) {
          node.particles.push(p);
          node.mass += 1;
          node.comX += p.x;
          node.comY += p.y;
          return;
        }
        
        if (node.children === null && node.particles.length === 0) {
          node.particles.push(p);
          node.mass = 1;
          node.comX = p.x;
          node.comY = p.y;
          return;
        }
        
        if (node.children === null && node.particles.length === 1) {
          // Subdivide
          const halfSize = node.size / 2;
          node.children = [
            createNode(node.x, node.y + halfSize, halfSize),           // NW
            createNode(node.x + halfSize, node.y + halfSize, halfSize), // NE
            createNode(node.x, node.y, halfSize),                       // SW
            createNode(node.x + halfSize, node.y, halfSize)             // SE
          ];
          
          // Re-insert existing particle
          const oldP = node.particles[0];
          node.particles = [];
          insertIntoChild(node, oldP, depth);
        }
        
        if (node.children !== null) {
          insertIntoChild(node, p, depth);
        }
        
        // Update mass and center of mass
        node.mass += 1;
        node.comX += p.x;
        node.comY += p.y;
      };
      
      const insertIntoChild = (node, p, depth) => {
        const halfSize = node.size / 2;
        const midX = node.x + halfSize;
        const midY = node.y + halfSize;
        
        let childIdx;
        if (p.x < midX) {
          childIdx = p.y < midY ? 2 : 0; // SW : NW
        } else {
          childIdx = p.y < midY ? 3 : 1; // SE : NE
        }
        insert(node.children[childIdx], p, depth + 1);
      };
      
      // Build tree
      for (const p of allParticles) {
        insert(root, p);
      }
      
      // Finalize centers of mass
      const finalize = (node) => {
        if (node.mass > 0) {
          node.comX /= node.mass;
          node.comY /= node.mass;
        }
        if (node.children) {
          for (const child of node.children) finalize(child);
        }
      };
      finalize(root);
      
      // Calculate force on a particle from a node
      const calcForce = (px, py, node, fx, fy) => {
        if (node.mass === 0) return [fx, fy];
        
        const dx = node.comX - px;
        const dy = node.comY - py;
        const dist2 = dx * dx + dy * dy;
        const dist = Math.sqrt(dist2 + gs2);
        
        // If leaf or sufficiently far, use approximation
        if (node.children === null || (node.size / dist) < qt) {
          // Check we're not computing self-force (same position)
          if (dist2 < 0.0001) return [fx, fy];
          
          const force = grav * node.mass / (dist2 + gs2);
          fx += force * dx / dist;
          fy += force * dy / dist;
        } else {
          // Recurse into children
          for (const child of node.children) {
            [fx, fy] = calcForce(px, py, child, fx, fy);
          }
        }
        return [fx, fy];
      };
      
      // Calculate forces for each real particle (not ghosts)
      for (const idx of liveParticles) {
        const px = posX[idx];
        const py = posY[idx];
        let [fx, fy] = calcForce(px, py, root, 0, 0);
        forceX[idx] = fx;
        forceY[idx] = fy;
      }
    }
    
    // Apply gravity to velocities with speed limit clamping
    for (const idx of liveParticles) {
      const oldSpeed = Math.sqrt(velX[idx] * velX[idx] + velY[idx] * velY[idx]);
      
      velX[idx] += forceX[idx];
      velY[idx] += forceY[idx];
      
      // Apply wind resistance (drag proportional to v²)
      if (dx > 0) {
        const vx = velX[idx];
        const dragForceX = dx * vx * Math.abs(vx);
        velX[idx] -= dragForceX;
      }
      if (dy > 0) {
        const vy = velY[idx];
        const dragForceY = dy * vy * Math.abs(vy);
        velY[idx] -= dragForceY;
      }
      
      const newSpeed = Math.sqrt(velX[idx] * velX[idx] + velY[idx] * velY[idx]);
      const effectiveLimit = Math.max(sl, oldSpeed);
      
      if (newSpeed > effectiveLimit && newSpeed > 0) {
        const scale = effectiveLimit / newSpeed;
        velX[idx] *= scale;
        velY[idx] *= scale;
      }
    }
    
    // ========== MOVEMENT PHASE ==========
    // Queues for blocked particles wanting to move to each cell
    const moveQueues = new Map();
    
    const tryMove = (idx) => {
      if (!alive[idx] || movedThisTick[idx]) return;
      
      const px = posX[idx];
      const py = posY[idx];
      let targetX = px + velX[idx];
      let targetY = py + velY[idx];
      
      // Wrap target position
      if (wrap) {
        [targetX, targetY] = wrapCoord(targetX, targetY, w, h, sx, sy);
      }
      
      const targetCellX = Math.floor(targetX);
      const targetCellY = Math.floor(targetY);
      
      // Check bounds if not wrapping
      if (!wrap && (targetCellX < 0 || targetCellX >= w || targetCellY < 0 || targetCellY >= h)) {
        movedThisTick[idx] = 1;
        return;
      }
      
      const targetIdx = targetCellY * w + targetCellX;
      const currentX = Math.floor(px);
      const currentY = Math.floor(py);
      const currentIdx = currentY * w + currentX;
      
      if (targetIdx === currentIdx) {
        // Just update position within same cell
        posX[idx] = targetX;
        posY[idx] = targetY;
        movedThisTick[idx] = 1;
        return;
      }
      
      if (alive[targetIdx]) {
        // Target occupied - queue up
        if (!moveQueues.has(targetIdx)) moveQueues.set(targetIdx, []);
        moveQueues.get(targetIdx).push({ idx, targetX, targetY, fromIdx: currentIdx });
        movedThisTick[idx] = 1; // Mark as processed (stuck)
        return;
      }
      
      // Move to target cell
      const oldIdx = currentIdx;
      alive[oldIdx] = 0;
      alive[targetIdx] = 1;
      posX[targetIdx] = targetX;
      posY[targetIdx] = targetY;
      velX[targetIdx] = velX[idx];
      velY[targetIdx] = velY[idx];
      colorH[targetIdx] = colorH[idx];
      colorS[targetIdx] = colorS[idx];
      colorL[targetIdx] = colorL[idx];
      movedThisTick[targetIdx] = 2; // Moved successfully
      
      // Clear old cell
      if (oldIdx !== targetIdx) {
        posX[oldIdx] = 0;
        posY[oldIdx] = 0;
        velX[oldIdx] = 0;
        velY[oldIdx] = 0;
        // Keep color for fading
      }
      
      // Check if anyone was waiting for the old cell
      if (moveQueues.has(oldIdx)) {
        const queue = moveQueues.get(oldIdx);
        if (queue.length > 0) {
          const next = queue.shift();
          // Recursively process cascade
          const cascadeMove = (item) => {
            const srcX = Math.floor(posX[item.idx]);
            const srcY = Math.floor(posY[item.idx]);
            const srcIdx = srcY * w + srcX;
            
            if (!alive[srcIdx]) return; // Already moved or dead
            
            const destIdx = item.fromIdx;
            if (alive[destIdx]) return; // Destination occupied again
            
            alive[srcIdx] = 0;
            alive[destIdx] = 1;
            posX[destIdx] = item.targetX;
            posY[destIdx] = item.targetY;
            velX[destIdx] = velX[item.idx];
            velY[destIdx] = velY[item.idx];
            colorH[destIdx] = colorH[item.idx];
            colorS[destIdx] = colorS[item.idx];
            colorL[destIdx] = colorL[item.idx];
            movedThisTick[destIdx] = 2;
            
            // Check cascade from srcIdx
            if (moveQueues.has(srcIdx)) {
              const nextQueue = moveQueues.get(srcIdx);
              if (nextQueue.length > 0) {
                cascadeMove(nextQueue.shift());
              }
            }
          };
          cascadeMove(next);
        }
      }
    };
    
    // Process movement in shuffled order
    for (const idx of cellOrder) {
      if (alive[idx]) {
        tryMove(idx);
      }
    }
    
    // ========== COLOR UPDATE PHASE ==========
    for (let i = 0; i < totalCells; i++) {
      if (alive[i]) {
        const speed = Math.sqrt(velX[i] * velX[i] + velY[i] * velY[i]);
        const angle = Math.atan2(velY[i], velX[i]) * 180 / Math.PI;
        colorH[i] = (angle + 360) % 360;
        
        // Saturation based on speed relative to speed limit
        const saturationSpeed = Math.min(speed / (sl * 0.75), 1.0);
        colorS[i] = 0.10 + saturationSpeed * 0.05; // OKLCH chroma
        
        // Dim if didn't move this tick
        colorL[i] = movedThisTick[i] === 2 ? 0.7 : 0.5;
      } else if (colorL[i] > 0) {
        // Fade dead cells
        colorL[i] = Math.max(0, colorL[i] * (1 - 1/fp) - 1/ff);
        colorS[i] = Math.max(0, colorS[i] * (1 - 1/fp) - 1/ff);
      }
    }
    
    setTick(t => t + 1);
  }, [PIPercent, PIAbsolute, PAPercent, PAAbsolute, RBPercent, RBAbsolute, RDPercent, RDAbsolute, RCPercent, RCAbsolute, getNeighborCoord, getWrappedCopies, wrapCoord]);
  
  // Default values for comparison (moved here so it's available to buildQueryString)
  const defaults = {
    W: 192, H: 120, RA: true, SX: 96, SY: 0,
    DN: 2/256 * 100,
    ivxn: -1.125, ivxx: 1.125, ivyn: -1.125, ivyx: 1.125,
    dragX: 0, dragY: 0,
    PI: null, PIp: null, PIa: 124,
    PA: null, PAp: null, PAa: 132,
    RB: null, RBp: null, RBa: 4,
    RD: null, RDp: null, RDa: 4,
    RC: null, RCp: null, RCa: 4,
    G: 0.5, GS: 2.5, SL: 2.25, UD: false, QT: 0.5,
    fps: 24,
    hue: 165, rotation: 0, FD: 75, FP: 6, FF: 127,
    zoom: false
  };
  
  // Build current query string for URL updates
  const buildQueryString = useCallback(() => {
    const params = [];
    
    if (W !== defaults.W) params.push(`W=${W}`);
    if (H !== defaults.H) params.push(`H=${H}`);
    if (wrapEnabled !== defaults.RA) params.push(`RA=${wrapEnabled}`);
    if (SX !== defaults.SX) params.push(`SX=${SX}`);
    if (SY !== defaults.SY) params.push(`SY=${SY}`);
    if (initialDensity !== defaults.DN) params.push(`DN=${initialDensity}`);
    if (initVelMinX !== defaults.ivxn) params.push(`ivxn=${initVelMinX}`);
    if (initVelMaxX !== defaults.ivxx) params.push(`ivxx=${initVelMaxX}`);
    if (initVelMinY !== defaults.ivyn) params.push(`ivyn=${initVelMinY}`);
    if (initVelMaxY !== defaults.ivyx) params.push(`ivyx=${initVelMaxY}`);
    if (dragX !== defaults.dragX) params.push(`dragX=${dragX}`);
    if (dragY !== defaults.dragY) params.push(`dragY=${dragY}`);
    
    if (PIPercent !== null) {
      if (PIPercent !== defaults.PIp) params.push(`PIp=${PIPercent}`);
    } else if (PIAbsolute !== null && PIAbsolute !== defaults.PIa) {
      params.push(`PI=${PIAbsolute}`);
    }
    
    if (PAPercent !== null) {
      if (PAPercent !== defaults.PAp) params.push(`PAp=${PAPercent}`);
    } else if (PAAbsolute !== null && PAAbsolute !== defaults.PAa) {
      params.push(`PA=${PAAbsolute}`);
    }
    
    if (RBPercent !== null) {
      if (RBPercent !== defaults.RBp) params.push(`RBp=${RBPercent}`);
    } else if (RBAbsolute !== null && RBAbsolute !== defaults.RBa) {
      params.push(`RB=${RBAbsolute}`);
    }
    
    if (RDPercent !== null) {
      if (RDPercent !== defaults.RDp) params.push(`RDp=${RDPercent}`);
    } else if (RDAbsolute !== null && RDAbsolute !== defaults.RDa) {
      params.push(`RD=${RDAbsolute}`);
    }
    
    if (RCPercent !== null) {
      if (RCPercent !== defaults.RCp) params.push(`RCp=${RCPercent}`);
    } else if (RCAbsolute !== null && RCAbsolute !== defaults.RCa) {
      params.push(`RC=${RCAbsolute}`);
    }
    
    if (G !== defaults.G) params.push(`G=${G}`);
    if (GS !== defaults.GS) params.push(`GS=${GS}`);
    if (SL !== defaults.SL) params.push(`SL=${SL}`);
    if (UD !== defaults.UD) params.push(`UD=${UD}`);
    if (QT !== defaults.QT) params.push(`QT=${QT}`);
    if (fps !== defaults.fps) params.push(`fps=${fps}`);
    
    if (hue !== defaults.hue) params.push(`hue=${hue}`);
    if (rotation !== defaults.rotation) params.push(`rotation=${rotation}`);
    if (FD !== defaults.FD) params.push(`FD=${FD}`);
    if (FP !== defaults.FP) params.push(`FP=${FP}`);
    if (FF !== defaults.FF) params.push(`FF=${FF}`);
    if (overlayOpen !== defaults.zoom) params.push(`zoom=${overlayOpen}`);
    
    return params.length > 0 ? '?' + params.join('&') : '';
  }, [W, H, wrapEnabled, SX, SY, initialDensity, initVelMinX, initVelMaxX, initVelMinY, initVelMaxY, dragX, dragY, PIPercent, PIAbsolute, PAPercent, PAAbsolute, RBPercent, RBAbsolute, RDPercent, RDAbsolute, RCPercent, RCAbsolute, G, GS, SL, UD, QT, fps, hue, rotation, FD, FP, FF, overlayOpen]);
  
  // Update URL if query string changed
  const updateUrlIfNeeded = useCallback(() => {
    const newQuery = buildQueryString();
    if (window.location.search !== newQuery) {
      const newUrl = window.location.pathname + newQuery;
      window.history.replaceState(null, '', newUrl);
    }
  }, [buildQueryString]);
  
  // Schedule URL update with 2s debounce
  const scheduleUrlUpdate = useCallback(() => {
    if (urlUpdateTimeoutRef.current) clearTimeout(urlUpdateTimeoutRef.current);
    urlUpdateTimeoutRef.current = setTimeout(() => {
      updateUrlIfNeeded();
    }, 2000);
  }, [updateUrlIfNeeded]);

  // Immediate pause
  const pauseImmediately = useCallback(() => {
    if (isFirstChangeInSequenceRef.current) {
      wasRunningBeforeChangeRef.current = running;
      isFirstChangeInSequenceRef.current = false;
    }
    setRunning(false);
    manualPauseResumeRef.current = false;
  }, [running]);
  
  const triggerRestart = useCallback(() => {
    if (settingsChangeTimeoutRef.current) clearTimeout(settingsChangeTimeoutRef.current);
    if (autoResumeTimeoutRef.current) clearTimeout(autoResumeTimeoutRef.current);
    
    clearGrid(settingsRef.current.W, settingsRef.current.H);
    setRenderTrigger(r => r + 1);
    
    settingsChangeTimeoutRef.current = setTimeout(() => {
      const s = settingsRef.current;
      initGridWithParams(s.W, s.H, initialDensity, s.initVelMinX, s.initVelMaxX, s.initVelMinY, s.initVelMaxY);
      setTick(0);
      setRenderTrigger(r => r + 1);
    }, 125);
    
    autoResumeTimeoutRef.current = setTimeout(() => {
      if (wasRunningBeforeChangeRef.current && !manualPauseResumeRef.current) {
        setCountdown(256);
        setRunning(true);
      }
      isFirstChangeInSequenceRef.current = true;
    }, 2000);
    
    scheduleUrlUpdate();
  }, [clearGrid, initGridWithParams, initialDensity, scheduleUrlUpdate]);
  
  const triggerResume = useCallback(() => {
    if (settingsChangeTimeoutRef.current) clearTimeout(settingsChangeTimeoutRef.current);
    if (autoResumeTimeoutRef.current) clearTimeout(autoResumeTimeoutRef.current);
    
    setRenderTrigger(r => r + 1);
    
    autoResumeTimeoutRef.current = setTimeout(() => {
      if (wasRunningBeforeChangeRef.current && !manualPauseResumeRef.current) {
        setCountdown(256);
        setRunning(true);
      }
      isFirstChangeInSequenceRef.current = true;
    }, 2000);
    
    scheduleUrlUpdate();
  }, [scheduleUrlUpdate]);
  
  // Animation loop
  useEffect(() => {
    if (!running || fps === 0) {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
        animationRef.current = null;
      }
      return;
    }
    
    const targetInterval = 1000 / fps;
    let lastStepTime = performance.now();
    
    const animate = (time) => {
      const elapsed = time - lastStepTime;
      if (elapsed >= targetInterval) {
        lastStepTime = time - (elapsed % targetInterval);
        step();
        render();
        recordFrameTime();
      }
      animationRef.current = requestAnimationFrame(animate);
    };
    
    animationRef.current = requestAnimationFrame(animate);
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
        animationRef.current = null;
      }
    };
  }, [running, fps, step, render, recordFrameTime]);
  
  // Countdown timer
  useEffect(() => {
    if (!running) {
      if (countdownRef.current) clearInterval(countdownRef.current);
      return;
    }
    countdownRef.current = setInterval(() => {
      setCountdown(c => {
        if (c <= 1) { setRunning(false); return 0; }
        return c - 1;
      });
    }, 1000);
    return () => { if (countdownRef.current) clearInterval(countdownRef.current); };
  }, [running]);
  
  // Initialize on mount - check URL for config
  useEffect(() => {
    const urlQuery = window.location.search;
    if (urlQuery && urlQuery.length > 1) {
      const queryString = urlQuery.slice(1); // Remove leading '?'
      const config = applyConfig(queryString);
      applyConfigToState(config);
      initGridWithParams(config.W, config.H, config.DN, config.ivxn, config.ivxx, config.ivyn, config.ivyx);
    } else {
      initGridWithParams(W, H, initialDensity, initVelMinX, initVelMaxX, initVelMinY, initVelMaxY);
    }
    setRenderTrigger(r => r + 1);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  
  useEffect(() => { render(); }, [renderTrigger, render]);
  useEffect(() => { render(); }, [tick, render]);
  
  // Viewport resize handling
  useEffect(() => {
    const handleResize = () => {
      setViewportWidth(window.innerWidth);
      setViewportHeight(window.innerHeight);
    };
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);
  
  const handleRestart = () => {
    manualPauseResumeRef.current = true;
    isFirstChangeInSequenceRef.current = true;
    if (settingsChangeTimeoutRef.current) clearTimeout(settingsChangeTimeoutRef.current);
    if (autoResumeTimeoutRef.current) clearTimeout(autoResumeTimeoutRef.current);
    frameTimesRef.current = [];
    initGridWithParams(W, H, initialDensity, initVelMinX, initVelMaxX, initVelMinY, initVelMaxY);
    setTick(0);
    setCountdown(256);
    setRunning(true);
    setRenderTrigger(r => r + 1);
    scheduleUrlUpdate();
  };
  
  const handleToggleRun = () => {
    manualPauseResumeRef.current = true;
    isFirstChangeInSequenceRef.current = true;
    if (settingsChangeTimeoutRef.current) clearTimeout(settingsChangeTimeoutRef.current);
    if (autoResumeTimeoutRef.current) clearTimeout(autoResumeTimeoutRef.current);
    if (!running) { setCountdown(256); frameTimesRef.current = []; }
    setRunning(!running);
  };
  
  const handleStep = () => {
    if (running) {
      manualPauseResumeRef.current = true;
      isFirstChangeInSequenceRef.current = true;
      if (settingsChangeTimeoutRef.current) clearTimeout(settingsChangeTimeoutRef.current);
      if (autoResumeTimeoutRef.current) clearTimeout(autoResumeTimeoutRef.current);
      setRunning(false);
    }
    step();
    render();
    recordFrameTime();
  };
  
  const handlePIChange = (percent, absolute) => {
    if (percent !== null) {
      setPIPercent(percent);
      setPIAbsolute(null);
      if (PAPercent !== null && PAPercent < percent) setPAPercent(percent);
    } else if (absolute !== null) {
      setPIAbsolute(absolute);
      setPIPercent(null);
      if (PAAbsolute !== null && PAAbsolute < absolute) setPAAbsolute(absolute);
    }
  };
  
  const handleCopyConfig = useCallback(() => {
    const params = [];
    
    if (W !== defaults.W) params.push(`W=${W}`);
    if (H !== defaults.H) params.push(`H=${H}`);
    if (wrapEnabled !== defaults.RA) params.push(`RA=${wrapEnabled}`);
    if (SX !== defaults.SX) params.push(`SX=${SX}`);
    if (SY !== defaults.SY) params.push(`SY=${SY}`);
    if (initialDensity !== defaults.DN) params.push(`DN=${initialDensity}`);
    if (initVelMinX !== defaults.ivxn) params.push(`ivxn=${initVelMinX}`);
    if (initVelMaxX !== defaults.ivxx) params.push(`ivxx=${initVelMaxX}`);
    if (initVelMinY !== defaults.ivyn) params.push(`ivyn=${initVelMinY}`);
    if (initVelMaxY !== defaults.ivyx) params.push(`ivyx=${initVelMaxY}`);
    if (dragX !== defaults.dragX) params.push(`dragX=${dragX}`);
    if (dragY !== defaults.dragY) params.push(`dragY=${dragY}`);
    
    // Population limits - only include active one
    if (PIPercent !== null) {
      if (PIPercent !== defaults.PIp) params.push(`PIp=${PIPercent}`);
    } else if (PIAbsolute !== null && PIAbsolute !== defaults.PIa) {
      params.push(`PI=${PIAbsolute}`);
    }
    
    if (PAPercent !== null) {
      if (PAPercent !== defaults.PAp) params.push(`PAp=${PAPercent}`);
    } else if (PAAbsolute !== null && PAAbsolute !== defaults.PAa) {
      params.push(`PA=${PAAbsolute}`);
    }
    
    // Rate limits
    if (RBPercent !== null) {
      if (RBPercent !== defaults.RBp) params.push(`RBp=${RBPercent}`);
    } else if (RBAbsolute !== null && RBAbsolute !== defaults.RBa) {
      params.push(`RB=${RBAbsolute}`);
    }
    
    if (RDPercent !== null) {
      if (RDPercent !== defaults.RDp) params.push(`RDp=${RDPercent}`);
    } else if (RDAbsolute !== null && RDAbsolute !== defaults.RDa) {
      params.push(`RD=${RDAbsolute}`);
    }
    
    if (RCPercent !== null) {
      if (RCPercent !== defaults.RCp) params.push(`RCp=${RCPercent}`);
    } else if (RCAbsolute !== null && RCAbsolute !== defaults.RCa) {
      params.push(`RC=${RCAbsolute}`);
    }
    
    // Physics
    if (G !== defaults.G) params.push(`G=${G}`);
    if (GS !== defaults.GS) params.push(`GS=${GS}`);
    if (SL !== defaults.SL) params.push(`SL=${SL}`);
    if (UD !== defaults.UD) params.push(`UD=${UD}`);
    if (QT !== defaults.QT) params.push(`QT=${QT}`);
    if (fps !== defaults.fps) params.push(`fps=${fps}`);
    
    // Cosmetic
    if (hue !== defaults.hue) params.push(`hue=${hue}`);
    if (rotation !== defaults.rotation) params.push(`rotation=${rotation}`);
    if (FD !== defaults.FD) params.push(`FD=${FD}`);
    if (FP !== defaults.FP) params.push(`FP=${FP}`);
    if (FF !== defaults.FF) params.push(`FF=${FF}`);
    if (overlayOpen !== defaults.zoom) params.push(`zoom=${overlayOpen}`);
    
    const queryString = params.length > 0 ? '?' + params.join('&') : '';
    const fullUrl = window.location.origin + window.location.pathname + queryString;
    navigator.clipboard.writeText(fullUrl);
  }, [W, H, wrapEnabled, SX, SY, initialDensity, initVelMinX, initVelMaxX, initVelMinY, initVelMaxY, dragX, dragY, PIPercent, PIAbsolute, PAPercent, PAAbsolute, RBPercent, RBAbsolute, RDPercent, RDAbsolute, RCPercent, RCAbsolute, G, GS, SL, UD, QT, fps, hue, rotation, FD, FP, FF, overlayOpen]);
  
  const parseConfigString = useCallback((text) => {
    // Extract query string from full URL or use as-is
    let queryString = text.trim();
    if (queryString.includes('?')) {
      queryString = queryString.split('?')[1] || '';
    }
    // Remove any hash
    if (queryString.includes('#')) {
      queryString = queryString.split('#')[0];
    }
    return queryString;
  }, []);
  
  const applyConfig = useCallback((queryString) => {
    const params = new URLSearchParams(queryString);
    const seen = new Set();
    
    // Check for duplicates
    const allKeys = [...params.keys()];
    const duplicates = allKeys.filter((key, i) => allKeys.indexOf(key) !== i);
    for (const dup of duplicates) {
      console.warn(`Partli config: duplicate parameter "${dup}", using first value`);
    }
    
    // Start with defaults
    let newW = defaults.W, newH = defaults.H, newRA = defaults.RA;
    let newSX = defaults.SX, newSY = defaults.SY, newDN = defaults.DN;
    let newIvxn = defaults.ivxn, newIvxx = defaults.ivxx;
    let newIvyn = defaults.ivyn, newIvyx = defaults.ivyx;
    let newDragX = defaults.dragX, newDragY = defaults.dragY;
    let newPIp = null, newPIa = defaults.PIa;
    let newPAp = null, newPAa = defaults.PAa;
    let newRBp = null, newRBa = defaults.RBa;
    let newRDp = null, newRDa = defaults.RDa;
    let newRCp = null, newRCa = defaults.RCa;
    let newG = defaults.G, newGS = defaults.GS, newSL = defaults.SL;
    let newUD = defaults.UD, newQT = defaults.QT;
    let newFps = defaults.fps;
    let newHue = defaults.hue, newRotation = defaults.rotation;
    let newFD = defaults.FD, newFP = defaults.FP, newFF = defaults.FF;
    let newZoom = defaults.zoom;
    
    for (const [key, value] of params) {
      if (seen.has(key)) continue;
      seen.add(key);
      
      const num = parseFloat(value);
      const bool = value === 'true' ? true : value === 'false' ? false : null;
      
      switch (key) {
        case 'W': if (!isNaN(num)) newW = Math.round(num); break;
        case 'H': if (!isNaN(num)) newH = Math.round(num); break;
        case 'RA': if (bool !== null) newRA = bool; break;
        case 'SX': if (!isNaN(num)) newSX = Math.round(num); break;
        case 'SY': if (!isNaN(num)) newSY = Math.round(num); break;
        case 'DN': if (!isNaN(num)) newDN = num; break;
        case 'ivxn': if (!isNaN(num)) newIvxn = num; break;
        case 'ivxx': if (!isNaN(num)) newIvxx = num; break;
        case 'ivyn': if (!isNaN(num)) newIvyn = num; break;
        case 'ivyx': if (!isNaN(num)) newIvyx = num; break;
        case 'dragX': if (!isNaN(num)) newDragX = num; break;
        case 'dragY': if (!isNaN(num)) newDragY = num; break;
        case 'PI': if (!isNaN(num)) { newPIa = Math.round(num); newPIp = null; } break;
        case 'PIp': if (!isNaN(num)) { newPIp = num; newPIa = null; } break;
        case 'PA': if (!isNaN(num)) { newPAa = Math.round(num); newPAp = null; } break;
        case 'PAp': if (!isNaN(num)) { newPAp = num; newPAa = null; } break;
        case 'RB': if (!isNaN(num)) { newRBa = Math.round(num); newRBp = null; } break;
        case 'RBp': if (!isNaN(num)) { newRBp = num; newRBa = null; } break;
        case 'RD': if (!isNaN(num)) { newRDa = Math.round(num); newRDp = null; } break;
        case 'RDp': if (!isNaN(num)) { newRDp = num; newRDa = null; } break;
        case 'RC': if (!isNaN(num)) { newRCa = Math.round(num); newRCp = null; } break;
        case 'RCp': if (!isNaN(num)) { newRCp = num; newRCa = null; } break;
        case 'G': if (!isNaN(num)) newG = num; break;
        case 'GS': if (!isNaN(num)) newGS = num; break;
        case 'SL': if (!isNaN(num)) newSL = num; break;
        case 'UD': if (bool !== null) newUD = bool; break;
        case 'QT': if (!isNaN(num)) newQT = num; break;
        case 'fps': if (!isNaN(num)) newFps = Math.round(num); break;
        case 'hue': if (!isNaN(num)) newHue = num; break;
        case 'rotation': if (!isNaN(num)) newRotation = num; break;
        case 'FD': if (!isNaN(num)) newFD = num; break;
        case 'FP': if (!isNaN(num)) newFP = num; break;
        case 'FF': if (!isNaN(num)) newFF = num; break;
        case 'zoom': if (bool !== null) newZoom = bool; break;
        default:
          console.warn(`Partli config: unknown parameter "${key}"`);
      }
    }
    
    return {
      W: newW, H: newH, RA: newRA, SX: newSX, SY: newSY, DN: newDN,
      ivxn: newIvxn, ivxx: newIvxx, ivyn: newIvyn, ivyx: newIvyx,
      dragX: newDragX, dragY: newDragY,
      PIp: newPIp, PIa: newPIa, PAp: newPAp, PAa: newPAa,
      RBp: newRBp, RBa: newRBa, RDp: newRDp, RDa: newRDa, RCp: newRCp, RCa: newRCa,
      G: newG, GS: newGS, SL: newSL, UD: newUD, QT: newQT, fps: newFps,
      hue: newHue, rotation: newRotation, FD: newFD, FP: newFP, FF: newFF,
      zoom: newZoom
    };
  }, []);
  
  const applyConfigToState = useCallback((config) => {
    setW(config.W);
    setH(config.H);
    setWrapEnabled(config.RA);
    setSX(config.SX);
    setSY(config.SY);
    setInitialDensity(config.DN);
    setInitVelMinX(config.ivxn);
    setInitVelMaxX(config.ivxx);
    setInitVelMinY(config.ivyn);
    setInitVelMaxY(config.ivyx);
    setDragX(config.dragX);
    setDragY(config.dragY);
    setPIPercent(config.PIp);
    setPIAbsolute(config.PIa);
    setPAPercent(config.PAp);
    setPAAbsolute(config.PAa);
    setRBPercent(config.RBp);
    setRBAbsolute(config.RBa);
    setRDPercent(config.RDp);
    setRDAbsolute(config.RDa);
    setRCPercent(config.RCp);
    setRCAbsolute(config.RCa);
    setG(config.G);
    setGS(config.GS);
    setSL(config.SL);
    setUD(config.UD);
    setQT(config.QT);
    setFps(config.fps);
    setHue(config.hue);
    setRotation(config.rotation);
    setFD(config.FD);
    setFP(config.FP);
    setFF(config.FF);
    setOverlayOpen(config.zoom);
  }, []);
  
  const handlePasteConfig = useCallback(async () => {
    try {
      const text = await navigator.clipboard.readText();
      const queryString = parseConfigString(text);
      const config = applyConfig(queryString);
      applyConfigToState(config);
      
      // Trigger restart with new config
      pauseImmediately();
      triggerRestart();
    } catch (err) {
      console.warn('Partli config: failed to read clipboard', err);
    }
  }, [parseConfigString, applyConfig, applyConfigToState, pauseImmediately, triggerRestart]);
  
  // Open overlay
  const handleCanvasClick = useCallback(() => {
    setOverlayOpen(true);
  }, []);
  
  // Close overlay
  const handleOverlayClick = useCallback(() => {
    setOverlayOpen(false);
    setTileUrl(null);
  }, []);
  
  // Update URL when overlay state changes (debounced)
  useEffect(() => {
    scheduleUrlUpdate();
  }, [overlayOpen, scheduleUrlUpdate]);
  
  // Render tile for overlay - matches canvas scale, with corner-based borders
  const renderOverlayTile = useCallback(() => {
    if (!overlayOpen) return;
    
    const alive = aliveRef.current;
    const colorH = colorHRef.current;
    const colorS = colorSRef.current;
    const colorL = colorLRef.current;
    const { W: w, H: h, SX: sx, SY: sy, hue: baseHue } = settingsRef.current;
    
    if (!alive) return;
    
    // Use same scale as main canvas would use for core grid
    const maxCoreDim = Math.max(w, h);
    const currentTargetSize = Math.min(
      viewportWidth - 2 * CONTENT_PADDING,
      viewportHeight / 2,
      MAX_TARGET_SIZE
    );
    
    let tileScale;
    if (maxCoreDim <= currentTargetSize) {
      tileScale = Math.max(1, Math.floor(currentTargetSize / maxCoreDim));
    } else {
      tileScale = currentTargetSize / maxCoreDim;
    }
    
    const tileW = Math.round(w * tileScale);
    const tileH = Math.round(h * tileScale);
    
    const offscreen = document.createElement('canvas');
    offscreen.width = tileW;
    offscreen.height = tileH;
    const ctx = offscreen.getContext('2d');
    const imageData = ctx.createImageData(tileW, tileH);
    const data = imageData.data;
    
    const deadL = 0;
    const deadC = 0;
    
    // Corner radius in display pixels
    const CORNER_RADIUS = 12;
    
    // Build list of all relevant corners (in grid coordinates)
    // These are corners of the core tile and corners of adjacent tiles that touch the core's perimeter
    const corners = [];
    
    // Core tile corners (in grid coords: 0,0 to w,h)
    corners.push([0, 0], [w, 0], [0, h], [w, h]);
    
    // Adjacent tile corners that land on core's edges due to stagger
    // Top edge (y=h): tiles above have their bottom edge here, offset by sx
    // Their corners are at x = sx, x = sx + w (mod w gives where they land on core's top edge)
    if (sx !== 0) {
      // Corners from tile above-right landing on top edge
      const topCornerX = sx % w;
      if (topCornerX !== 0) {
        corners.push([topCornerX, h]);
      }
      // And the wrapped one
      const topCornerX2 = (sx + w) % w;
      if (topCornerX2 !== 0 && topCornerX2 !== topCornerX) {
        corners.push([topCornerX2, h]);
      }
      // Bottom edge (y=0): tiles below
      corners.push([topCornerX, 0]);
      if (topCornerX2 !== 0 && topCornerX2 !== topCornerX) {
        corners.push([topCornerX2, 0]);
      }
    }
    
    if (sy !== 0) {
      // Corners from tile to the right landing on right edge
      const rightCornerY = sy % h;
      if (rightCornerY !== 0) {
        corners.push([w, rightCornerY]);
      }
      const rightCornerY2 = (sy + h) % h;
      if (rightCornerY2 !== 0 && rightCornerY2 !== rightCornerY) {
        corners.push([w, rightCornerY2]);
      }
      // Left edge (x=0): tiles to the left
      corners.push([0, rightCornerY]);
      if (rightCornerY2 !== 0 && rightCornerY2 !== rightCornerY) {
        corners.push([0, rightCornerY2]);
      }
    }
    
    // Convert corners to display pixel coordinates
    const cornersPx = corners.map(([cx, cy]) => [cx * tileScale, (h - cy) * tileScale]);
    
    // Check if a display pixel is near any corner
    const isNearCorner = (px, py) => {
      for (const [cx, cy] of cornersPx) {
        const dx = px - cx;
        const dy = py - cy;
        if (dx * dx + dy * dy <= CORNER_RADIUS * CORNER_RADIUS) {
          return true;
        }
      }
      return false;
    };
    
    // Render each grid cell
    for (let gy = 0; gy < h; gy++) {
      for (let gx = 0; gx < w; gx++) {
        const srcIdx = gy * w + gx;
        
        let cellH = baseHue, cellC = deadC, cellL = deadL;
        
        if (alive[srcIdx]) {
          cellH = (colorH[srcIdx] + baseHue) % 360;
          cellC = colorS[srcIdx];
          cellL = colorL[srcIdx];
        } else if (colorL[srcIdx] > 0) {
          cellH = (colorH[srcIdx] + baseHue) % 360;
          cellC = colorS[srcIdx] * 0.5;
          cellL = colorL[srcIdx];
        }
        
        const [baseR, baseG, baseB] = oklchToRgb(cellL, cellC, cellH);
        
        // Check if this cell is on the inside edge
        const isInsideEdge = gx === 0 || gx === w - 1 || gy === 0 || gy === h - 1;
        
        // Calculate pixel range for this cell
        const pxStartX = Math.round(gx * tileScale);
        const pxEndX = Math.round((gx + 1) * tileScale);
        const pxStartY = Math.round((h - 1 - gy) * tileScale);
        const pxEndY = Math.round((h - gy) * tileScale);
        
        for (let py = pxStartY; py < pxEndY; py++) {
          for (let px = pxStartX; px < pxEndX; px++) {
            if (px < 0 || px >= tileW || py < 0 || py >= tileH) continue;
            
            let r = baseR, g = baseG, b = baseB;
            
            // Apply corner-based lightening for inside edges
            if (isInsideEdge && isNearCorner(px, py)) {
              r = r + (255 - r) * 0.125;
              g = g + (255 - g) * 0.125;
              b = b + (255 - b) * 0.125;
            }
            
            const dstIdx = (py * tileW + px) * 4;
            data[dstIdx] = Math.round(r);
            data[dstIdx + 1] = Math.round(g);
            data[dstIdx + 2] = Math.round(b);
            data[dstIdx + 3] = 255;
          }
        }
      }
    }
    
    ctx.putImageData(imageData, 0, 0);
    
    // Preload image before setting URL to prevent flicker
    const newUrl = offscreen.toDataURL('image/png');
    const img = new Image();
    img.onload = () => {
      setTileUrl(newUrl);
    };
    img.src = newUrl;
  }, [overlayOpen, viewportWidth, viewportHeight]);
  
  // Update overlay tile when tick changes and overlay is open
  useEffect(() => {
    if (overlayOpen) {
      renderOverlayTile();
    }
  }, [overlayOpen, tick, renderOverlayTile, viewportWidth, viewportHeight]);

  // Canvas sizing - responsive to viewport
  const targetSize = Math.min(
    viewportWidth - 2 * CONTENT_PADDING,
    viewportHeight / 2,
    MAX_TARGET_SIZE
  );
  const canvasW = wrapEnabled ? W * 2 : W;
  const canvasH = wrapEnabled ? H * 2 : H;
  const maxCanvasDim = Math.max(canvasW, canvasH);
  
  // Calculate rotated bounding box dimensions
  const radians = rotation * Math.PI / 180;
  const cosR = Math.abs(Math.cos(radians));
  const sinR = Math.abs(Math.sin(radians));
  
  let displayW, displayH, imageRendering;
  if (maxCanvasDim <= targetSize) {
    // Scale up by integer multiple
    const integerScale = Math.floor(targetSize / maxCanvasDim);
    const scale = Math.max(1, integerScale);
    displayW = canvasW * scale;
    displayH = canvasH * scale;
    imageRendering = 'pixelated';
    
    // Check if rotated bounding box exceeds targetSize, scale down if needed
    const rotatedW = displayW * cosR + displayH * sinR;
    const rotatedH = displayW * sinR + displayH * cosR;
    const maxRotatedDim = Math.max(rotatedW, rotatedH);
    if (maxRotatedDim > targetSize) {
      const shrinkScale = targetSize / maxRotatedDim;
      displayW *= shrinkScale;
      displayH *= shrinkScale;
      imageRendering = 'auto'; // No longer integer-scaled
    }
  } else {
    // Scale down to fit
    const scale = targetSize / maxCanvasDim;
    displayW = canvasW * scale;
    displayH = canvasH * scale;
    imageRendering = 'auto';
    
    // Check if rotated bounding box exceeds targetSize, scale down further if needed
    const rotatedW = displayW * cosR + displayH * sinR;
    const rotatedH = displayW * sinR + displayH * cosR;
    const maxRotatedDim = Math.max(rotatedW, rotatedH);
    if (maxRotatedDim > targetSize) {
      const shrinkScale = targetSize / maxRotatedDim;
      displayW *= shrinkScale;
      displayH *= shrinkScale;
    }
  }
  
  // Container size based on rotated bounding box
  const rotatedDisplayW = displayW * cosR + displayH * sinR;
  const rotatedDisplayH = displayW * sinR + displayH * cosR;
  const containerSize = Math.max(rotatedDisplayW, rotatedDisplayH);
  
  // Population count
  let popCount = 0;
  if (aliveRef.current) {
    for (let i = 0; i < aliveRef.current.length; i++) if (aliveRef.current[i]) popCount++;
  }
  
  const totalCells = W * H;
  const popPercent = totalCells > 0 ? (popCount / totalCells * 100).toFixed(2) : '0.00';
  
  const tickStr = tick.toString();
  if (tickStr.length > maxTickWidthRef.current) maxTickWidthRef.current = tickStr.length;
  const popStr = popCount.toString();
  if (popStr.length > maxPopWidthRef.current) maxPopWidthRef.current = popStr.length;
  const fpsStr = actualFps.toString();
  if (fpsStr.length > maxFpsWidthRef.current) maxFpsWidthRef.current = fpsStr.length;
  const countdownStr = countdown.toString();
  if (countdownStr.length > maxPauseWidthRef.current) maxPauseWidthRef.current = countdownStr.length;
  const aspectStr = getAspectRatio();
  if (aspectStr.length > maxAspectWidthRef.current) maxAspectWidthRef.current = aspectStr.length;
  
  const padNum = (num, width) => num.toString().padStart(width, '\u00A0');
  const padStr = (str, width) => str.padStart(width, '\u00A0');
  
  let fpsColor = '#aaa';
  if (fps > 0 && actualFps > 0) {
    if (actualFps < fps * 3 / 4) fpsColor = '#f44';
    else if (actualFps > fps * 5 / 4) fpsColor = '#f4f';
  }
  
  const maxAbsoluteVal = W * H;
  
  // Calculate the height of the fixed canvas area based on actual rotated canvas height
  const canvasAreaHeight = rotatedDisplayH + CONTENT_PADDING * 2;

  return (
    <div style={{ 
      backgroundColor: '#1a1a2e', 
      color: '#eee', 
      minHeight: '100vh',
      width: '100vw',
      fontFamily: 'system-ui, sans-serif',
      boxSizing: 'border-box',
      overflowX: 'hidden',
      overflowY: 'auto'
    }}>
      {/* Fixed canvas - transparent container, only canvas is opaque */}
      <div style={{ 
        position: 'fixed',
        top: CONTENT_PADDING,
        left: 0,
        right: 0,
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'flex-start',
        pointerEvents: 'none',
        zIndex: 10
      }}>
        <div style={{ 
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          width: rotatedDisplayW,
          height: rotatedDisplayH,
          overflow: 'visible'
        }}>
          <canvas
            ref={canvasRef}
            width={canvasW}
            height={canvasH}
            onClick={handleCanvasClick}
            onMouseEnter={() => setCanvasHover(true)}
            onMouseLeave={() => setCanvasHover(false)}
            style={{
              width: displayW,
              height: displayH,
              imageRendering,
              border: `1px solid ${canvasHover ? '#40e0d0' : '#bfbfbf'}`,
              backgroundColor: '#000',
              cursor: 'pointer',
              pointerEvents: 'auto',
              transform: rotation !== 0 ? `rotate(${rotation}deg)` : undefined,
              transformOrigin: 'center'
            }}
          />
        </div>
      </div>
      
      {/* Content - starts below canvas area, scrolls up behind it */}
      <div style={{
        marginTop: canvasAreaHeight,
        borderTop: '1px solid #444',
        padding: CONTENT_PADDING,
        maxWidth: '400px',
        marginLeft: 'auto',
        marginRight: 'auto'
      }}>
        {/* Status line */}
        <div style={{ fontSize: '14px', color: '#aaa' }}>
          <span>Tick: </span>
          <span style={{ fontFamily: 'monospace' }}>{padNum(tick, maxTickWidthRef.current)}</span>
          <span> | FPS: </span>
          <span style={{ fontFamily: 'monospace', color: fpsColor }}>{padNum(actualFps, maxFpsWidthRef.current)}</span>
          <span> | Pop: </span>
          <span style={{ fontFamily: 'monospace' }}>{padNum(popCount, maxPopWidthRef.current)}</span>
          <span> (</span>
          <span style={{ fontFamily: 'monospace' }}>{popPercent.padStart(7, '\u00A0')}</span>
          <span>%) | Aspect: </span>
          <span style={{ fontFamily: 'monospace' }}>{padStr(aspectStr, maxAspectWidthRef.current)}</span>
          {running && (
            <>
              <span> | Pause in: </span>
              <span style={{ fontFamily: 'monospace' }}>{padNum(countdown, maxPauseWidthRef.current)}s</span>
            </>
          )}
        </div>
        
        {/* Buttons */}
        <div style={{ marginTop: '12px', display: 'flex', gap: '8px', flexWrap: 'wrap', position: 'relative', zIndex: 1 }}>
          <button onClick={handleToggleRun}
            style={{ padding: '8px 16px', width: '80px', backgroundColor: running ? '#c44' : '#4a4', border: 'none', borderRadius: '4px', color: 'white', cursor: 'pointer' }}>
            {running ? 'Pause' : 'Run'}
          </button>
          <button onClick={handleStep}
            style={{ padding: '8px 16px', width: '80px', backgroundColor: '#555', border: 'none', borderRadius: '4px', color: 'white', cursor: 'pointer' }}>
            Step
          </button>
          <button onClick={handleRestart}
            style={{ padding: '8px 16px', width: '80px', backgroundColor: '#555', border: 'none', borderRadius: '4px', color: 'white', cursor: 'pointer' }}>
            Restart
          </button>
          <button onClick={handleCopyConfig}
            style={{ padding: '8px 16px', width: '80px', backgroundColor: '#555', border: 'none', borderRadius: '4px', color: 'white', cursor: 'pointer' }}>
            Copy
          </button>
          <button onClick={handlePasteConfig}
            style={{ padding: '8px 16px', width: '80px', backgroundColor: '#555', border: 'none', borderRadius: '4px', color: 'white', cursor: 'pointer' }}>
            Paste
          </button>
        </div>
        
        {/* Simulation */}
        <h3 style={{ margin: '16px 0 12px 0', color: '#8cf' }}>Simulation</h3>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Frame Rate: <EditableValue {...evProps} fieldName="fps" value={fps} displayValue={`${fps} FPS`} onCommit={v => setFps(Math.round(Math.max(0, Math.min(120, v))))} />
          <input type="range" min="0" max="120" value={fps}
            onChange={e => setFps(Number(e.target.value))}
            style={{ width: '100%' }}
          />
        </label>
        
        {/* Grid Settings */}
        <h3 style={{ margin: '16px 0 12px 0', color: '#8cf' }}>Grid Settings</h3>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Width (W): <EditableValue {...evProps} fieldName="W" value={W} onCommit={v => { const newW = Math.round(Math.max(16, Math.min(1024, v))); const { sx, sy } = calculateDefaultStagger(newW, H); setW(newW); setSX(sx); setSY(sy); pauseImmediately(); triggerRestart(); }} />
          <input type="range" min="16" max="1024" value={W} 
            onChange={e => {
              const newW = Number(e.target.value);
              const { sx, sy } = calculateDefaultStagger(newW, H);
              setW(newW); setSX(sx); setSY(sy);
              pauseImmediately();
            }}
            onMouseUp={triggerRestart}
            onTouchEnd={triggerRestart}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Height (H): <EditableValue {...evProps} fieldName="H" value={H} onCommit={v => { const newH = Math.round(Math.max(16, Math.min(1024, v))); const { sx, sy } = calculateDefaultStagger(W, newH); setH(newH); setSX(sx); setSY(sy); pauseImmediately(); triggerRestart(); }} />
          <input type="range" min="16" max="1024" value={H}
            onChange={e => {
              const newH = Number(e.target.value);
              const { sx, sy } = calculateDefaultStagger(W, newH);
              setH(newH); setSX(sx); setSY(sy);
              pauseImmediately();
            }}
            onMouseUp={triggerRestart}
            onTouchEnd={triggerRestart}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '12px' }}>
          <input type="checkbox" checked={wrapEnabled} 
            onChange={e => { setWrapEnabled(e.target.checked); pauseImmediately(); triggerResume(); }} />
          Enable Wrapping (RA)
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px', opacity: wrapEnabled ? 1 : 0.4 }}>
          Stagger X (SX): <EditableValue {...evProps} fieldName="SX" value={SX} onCommit={v => { const val = Math.round(Math.max(0, Math.min(W, v))); setSX(val); if (val !== 0) setSY(0); pauseImmediately(); triggerResume(); setRenderTrigger(r => r + 1); }} />
          <input type="range" min="0" max={W} value={SX}
            onChange={e => { 
              const v = Number(e.target.value);
              setSX(v); if (v !== 0) setSY(0);
              pauseImmediately();
              setRenderTrigger(r => r + 1);
            }}
            onMouseUp={triggerResume}
            onTouchEnd={triggerResume}
            disabled={!wrapEnabled}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px', opacity: wrapEnabled ? 1 : 0.4 }}>
          Stagger Y (SY): <EditableValue {...evProps} fieldName="SY" value={SY} onCommit={v => { const val = Math.round(Math.max(0, Math.min(H, v))); setSY(val); if (val !== 0) setSX(0); pauseImmediately(); triggerResume(); setRenderTrigger(r => r + 1); }} />
          <input type="range" min="0" max={H} value={SY}
            onChange={e => { 
              const v = Number(e.target.value);
              setSY(v); if (v !== 0) setSX(0);
              pauseImmediately();
              setRenderTrigger(r => r + 1);
            }}
            onMouseUp={triggerResume}
            onTouchEnd={triggerResume}
            disabled={!wrapEnabled}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Initial Density (DN): <EditableValue {...evProps} fieldName="DN" value={initialDensity} displayValue={`${initialDensity.toFixed(2)}%`} onCommit={v => { setInitialDensity(Math.max(0, Math.min(100, v))); pauseImmediately(); triggerRestart(); }} />
          <input type="range" min="0" max="100" step="0.01" value={initialDensity}
            onChange={e => { setInitialDensity(Number(e.target.value)); pauseImmediately(); }}
            onMouseUp={triggerRestart}
            onTouchEnd={triggerRestart}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Init Vel Min X: <EditableValue {...evProps} fieldName="ivxn" value={initVelMinX} displayValue={initVelMinX.toFixed(2)} onCommit={v => { setInitVelMinX(Math.max(-SL, Math.min(SL, v))); pauseImmediately(); triggerRestart(); }} />
          <input type="range" min={-SL} max={SL} step="0.01" value={initVelMinX}
            onChange={e => { setInitVelMinX(Number(e.target.value)); pauseImmediately(); }}
            onMouseUp={triggerRestart}
            onTouchEnd={triggerRestart}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Init Vel Max X: <EditableValue {...evProps} fieldName="ivxx" value={initVelMaxX} displayValue={initVelMaxX.toFixed(2)} onCommit={v => { setInitVelMaxX(Math.max(-SL, Math.min(SL, v))); pauseImmediately(); triggerRestart(); }} />
          <input type="range" min={-SL} max={SL} step="0.01" value={initVelMaxX}
            onChange={e => { setInitVelMaxX(Number(e.target.value)); pauseImmediately(); }}
            onMouseUp={triggerRestart}
            onTouchEnd={triggerRestart}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Init Vel Min Y: <EditableValue {...evProps} fieldName="ivyn" value={initVelMinY} displayValue={initVelMinY.toFixed(2)} onCommit={v => { setInitVelMinY(Math.max(-SL, Math.min(SL, v))); pauseImmediately(); triggerRestart(); }} />
          <input type="range" min={-SL} max={SL} step="0.01" value={initVelMinY}
            onChange={e => { setInitVelMinY(Number(e.target.value)); pauseImmediately(); }}
            onMouseUp={triggerRestart}
            onTouchEnd={triggerRestart}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Init Vel Max Y: <EditableValue {...evProps} fieldName="ivyx" value={initVelMaxY} displayValue={initVelMaxY.toFixed(2)} onCommit={v => { setInitVelMaxY(Math.max(-SL, Math.min(SL, v))); pauseImmediately(); triggerRestart(); }} />
          <input type="range" min={-SL} max={SL} step="0.01" value={initVelMaxY}
            onChange={e => { setInitVelMaxY(Number(e.target.value)); pauseImmediately(); }}
            onMouseUp={triggerRestart}
            onTouchEnd={triggerRestart}
            style={{ width: '100%' }}
          />
        </label>
        
        {/* Particle Physics */}
        <h3 style={{ margin: '16px 0 12px 0', color: '#8cf' }}>Particle Physics</h3>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Gravity (G): <EditableValue {...evProps} fieldName="G" value={G} displayValue={G.toFixed(3)} onCommit={v => { setG(Math.max(0, Math.min(1, v))); pauseImmediately(); triggerResume(); }} />
          <input type="range" min="0" max="1" step="0.001" value={G}
            onChange={e => { setG(Number(e.target.value)); pauseImmediately(); }}
            onMouseUp={triggerResume}
            onTouchEnd={triggerResume}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Gravity Softening (GS): <EditableValue {...evProps} fieldName="GS" value={GS} displayValue={GS.toFixed(2)} onCommit={v => { setGS(Math.max(0.1, Math.min(10, v))); pauseImmediately(); triggerResume(); }} />
          <input type="range" min="0.1" max="10" step="0.1" value={GS}
            onChange={e => { setGS(Number(e.target.value)); pauseImmediately(); }}
            onMouseUp={triggerResume}
            onTouchEnd={triggerResume}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Speed Limit (SL): <EditableValue {...evProps} fieldName="SL" value={SL} displayValue={SL.toFixed(2)} onCommit={v => { setSL(Math.max(0.5, Math.min(20, v))); pauseImmediately(); triggerResume(); }} />
          <input type="range" min="0.5" max="20" step="0.1" value={SL}
            onChange={e => { setSL(Number(e.target.value)); pauseImmediately(); }}
            onMouseUp={triggerResume}
            onTouchEnd={triggerResume}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '12px' }}>
          <input type="checkbox" checked={UD} 
            onChange={e => { setUD(e.target.checked); pauseImmediately(); triggerResume(); }} />
          Unconserved Deaths (UD)
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Approximation (QT): <EditableValue {...evProps} fieldName="QT" value={QT} displayValue={QT === 0 ? 'Exact' : QT.toFixed(2)} onCommit={v => { setQT(Math.max(0, Math.min(2, v))); pauseImmediately(); triggerResume(); }} />
          <input type="range" min="0" max="2" step="0.05" value={QT}
            onChange={e => { setQT(Number(e.target.value)); pauseImmediately(); }}
            onMouseUp={triggerResume}
            onTouchEnd={triggerResume}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Drag X: <EditableValue {...evProps} fieldName="dragX" value={dragX} displayValue={dragX.toFixed(3)} onCommit={v => { setDragX(Math.max(0, Math.min(1, v))); pauseImmediately(); triggerResume(); }} />
          <input type="range" min="0" max="1" step="0.001" value={dragX}
            onChange={e => { setDragX(Number(e.target.value)); pauseImmediately(); }}
            onMouseUp={triggerResume}
            onTouchEnd={triggerResume}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Drag Y: <EditableValue {...evProps} fieldName="dragY" value={dragY} displayValue={dragY.toFixed(3)} onCommit={v => { setDragY(Math.max(0, Math.min(1, v))); pauseImmediately(); triggerResume(); }} />
          <input type="range" min="0" max="1" step="0.001" value={dragY}
            onChange={e => { setDragY(Number(e.target.value)); pauseImmediately(); }}
            onMouseUp={triggerResume}
            onTouchEnd={triggerResume}
            style={{ width: '100%' }}
          />
        </label>
        
        {/* Population Limits */}
        <h3 style={{ margin: '16px 0 12px 0', color: '#8cf' }}>Population Limits</h3>
        
        <div style={{ marginBottom: '16px' }}>
          <div style={{ marginBottom: '4px', color: '#ccc' }}>Min Population (PI)</div>
          <div style={{ display: 'flex', gap: '12px' }}>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: '12px', color: '#888' }}>%: <EditableValue {...evProps} fieldName="PIp" value={PIPercent} displayValue={PIPercent !== null ? PIPercent.toFixed(1) : 'N/A'} onCommit={v => { handlePIChange(Math.max(0, Math.min(50, v)), null); pauseImmediately(); triggerResume(); }} /></div>
              <input type="range" min="0" max="50" step="0.5"
                value={PIPercent !== null ? PIPercent : 0}
                onChange={e => {
                  const v = Number(e.target.value);
                  if (v > 0 || PIAbsolute === null) { handlePIChange(v, null); pauseImmediately(); }
                }}
                onMouseUp={triggerResume}
                onTouchEnd={triggerResume}
                style={{ width: '100%' }}
              />
            </div>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: '12px', color: '#888' }}>#: <EditableValue {...evProps} fieldName="PIa" value={PIAbsolute} displayValue={PIAbsolute !== null ? PIAbsolute : 'N/A'} onCommit={v => { handlePIChange(null, Math.round(Math.max(0, Math.min(maxAbsoluteVal, v)))); pauseImmediately(); triggerResume(); }} /></div>
              <input type="range" min="0" max={maxAbsoluteVal}
                value={PIAbsolute !== null ? PIAbsolute : 0}
                onChange={e => {
                  const v = Number(e.target.value);
                  if (v > 0 || PIPercent === null) { handlePIChange(null, v); pauseImmediately(); }
                }}
                onMouseUp={triggerResume}
                onTouchEnd={triggerResume}
                style={{ width: '100%' }}
              />
            </div>
          </div>
        </div>
        
        <div style={{ marginBottom: '16px' }}>
          <div style={{ marginBottom: '4px', color: '#ccc' }}>Max Population (PA)</div>
          <div style={{ display: 'flex', gap: '12px' }}>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: '12px', color: '#888' }}>%: <EditableValue {...evProps} fieldName="PAp" value={PAPercent} displayValue={PAPercent !== null ? PAPercent.toFixed(1) : 'N/A'} onCommit={v => { setPAPercent(Math.max(0, Math.min(100, v))); setPAAbsolute(null); pauseImmediately(); triggerResume(); }} /></div>
              <input type="range" min="0" max="100" step="0.5"
                value={PAPercent !== null ? PAPercent : 0}
                onChange={e => {
                  const v = Number(e.target.value);
                  if (v > 0 || PAAbsolute === null) { setPAPercent(v); setPAAbsolute(null); pauseImmediately(); }
                }}
                onMouseUp={triggerResume}
                onTouchEnd={triggerResume}
                style={{ width: '100%' }}
              />
            </div>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: '12px', color: '#888' }}>#: <EditableValue {...evProps} fieldName="PAa" value={PAAbsolute} displayValue={PAAbsolute !== null ? PAAbsolute : 'N/A'} onCommit={v => { setPAAbsolute(Math.round(Math.max(0, Math.min(maxAbsoluteVal, v)))); setPAPercent(null); pauseImmediately(); triggerResume(); }} /></div>
              <input type="range" min="0" max={maxAbsoluteVal}
                value={PAAbsolute !== null ? PAAbsolute : 0}
                onChange={e => {
                  const v = Number(e.target.value);
                  if (v > 0 || PAPercent === null) { setPAAbsolute(v); setPAPercent(null); pauseImmediately(); }
                }}
                onMouseUp={triggerResume}
                onTouchEnd={triggerResume}
                style={{ width: '100%' }}
              />
            </div>
          </div>
        </div>
        
        {/* Rate Limits */}
        <h3 style={{ margin: '16px 0 12px 0', color: '#8cf' }}>Rate Limits</h3>
        
        <div style={{ marginBottom: '16px' }}>
          <div style={{ marginBottom: '4px', color: '#ccc' }}>Birth Rate (RB)</div>
          <div style={{ display: 'flex', gap: '12px' }}>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: '12px', color: '#888' }}>%: <EditableValue {...evProps} fieldName="RBp" value={RBPercent} displayValue={RBPercent !== null ? RBPercent.toFixed(1) : 'N/A'} onCommit={v => { setRBPercent(Math.max(0, Math.min(100, v))); setRBAbsolute(null); if (RDPercent !== null) { setRCPercent(v + RDPercent); setRCAbsolute(null); } pauseImmediately(); triggerResume(); }} /></div>
              <input type="range" min="0" max="100" step="0.5"
                value={RBPercent !== null ? RBPercent : 0}
                onChange={e => {
                  const v = Number(e.target.value);
                  if (v > 0 || RBAbsolute === null) {
                    setRBPercent(v); setRBAbsolute(null);
                    if (RDPercent !== null) { setRCPercent(v + RDPercent); setRCAbsolute(null); }
                    pauseImmediately();
                  }
                }}
                onMouseUp={triggerResume}
                onTouchEnd={triggerResume}
                style={{ width: '100%' }}
              />
            </div>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: '12px', color: '#888' }}>#: <EditableValue {...evProps} fieldName="RBa" value={RBAbsolute} displayValue={RBAbsolute !== null ? RBAbsolute : 'N/A'} onCommit={v => { setRBAbsolute(Math.round(Math.max(0, Math.min(maxAbsoluteVal, v)))); setRBPercent(null); if (RDAbsolute !== null) { setRCAbsolute(v + RDAbsolute); setRCPercent(null); } pauseImmediately(); triggerResume(); }} /></div>
              <input type="range" min="0" max={maxAbsoluteVal}
                value={RBAbsolute !== null ? RBAbsolute : 0}
                onChange={e => {
                  const v = Number(e.target.value);
                  if (v > 0 || RBPercent === null) {
                    setRBAbsolute(v); setRBPercent(null);
                    if (RDAbsolute !== null) { setRCAbsolute(v + RDAbsolute); setRCPercent(null); }
                    pauseImmediately();
                  }
                }}
                onMouseUp={triggerResume}
                onTouchEnd={triggerResume}
                style={{ width: '100%' }}
              />
            </div>
          </div>
        </div>
        
        <div style={{ marginBottom: '16px' }}>
          <div style={{ marginBottom: '4px', color: '#ccc' }}>Death Rate (RD)</div>
          <div style={{ display: 'flex', gap: '12px' }}>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: '12px', color: '#888' }}>%: <EditableValue {...evProps} fieldName="RDp" value={RDPercent} displayValue={RDPercent !== null ? RDPercent.toFixed(1) : 'N/A'} onCommit={v => { setRDPercent(Math.max(0, Math.min(100, v))); setRDAbsolute(null); if (RBPercent !== null) { setRCPercent(RBPercent + v); setRCAbsolute(null); } pauseImmediately(); triggerResume(); }} /></div>
              <input type="range" min="0" max="100" step="0.5"
                value={RDPercent !== null ? RDPercent : 0}
                onChange={e => {
                  const v = Number(e.target.value);
                  if (v > 0 || RDAbsolute === null) {
                    setRDPercent(v); setRDAbsolute(null);
                    if (RBPercent !== null) { setRCPercent(RBPercent + v); setRCAbsolute(null); }
                    pauseImmediately();
                  }
                }}
                onMouseUp={triggerResume}
                onTouchEnd={triggerResume}
                style={{ width: '100%' }}
              />
            </div>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: '12px', color: '#888' }}>#: <EditableValue {...evProps} fieldName="RDa" value={RDAbsolute} displayValue={RDAbsolute !== null ? RDAbsolute : 'N/A'} onCommit={v => { setRDAbsolute(Math.round(Math.max(0, Math.min(maxAbsoluteVal, v)))); setRDPercent(null); if (RBAbsolute !== null) { setRCAbsolute(RBAbsolute + v); setRCPercent(null); } pauseImmediately(); triggerResume(); }} /></div>
              <input type="range" min="0" max={maxAbsoluteVal}
                value={RDAbsolute !== null ? RDAbsolute : 0}
                onChange={e => {
                  const v = Number(e.target.value);
                  if (v > 0 || RDPercent === null) {
                    setRDAbsolute(v); setRDPercent(null);
                    if (RBAbsolute !== null) { setRCAbsolute(RBAbsolute + v); setRCPercent(null); }
                    pauseImmediately();
                  }
                }}
                onMouseUp={triggerResume}
                onTouchEnd={triggerResume}
                style={{ width: '100%' }}
              />
            </div>
          </div>
        </div>
        
        <div style={{ marginBottom: '16px' }}>
          <div style={{ marginBottom: '4px', color: '#ccc' }}>Change Rate (RC)</div>
          <div style={{ display: 'flex', gap: '12px' }}>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: '12px', color: '#888' }}>%: <EditableValue {...evProps} fieldName="RCp" value={RCPercent} displayValue={RCPercent !== null ? RCPercent.toFixed(1) : 'N/A'} onCommit={v => { setRCPercent(Math.max(0, Math.min(200, v))); setRCAbsolute(null); pauseImmediately(); triggerResume(); }} /></div>
              <input type="range" min="0" max="200" step="0.5"
                value={RCPercent !== null ? RCPercent : 0}
                onChange={e => {
                  const v = Number(e.target.value);
                  if (v > 0 || RCAbsolute === null) { setRCPercent(v); setRCAbsolute(null); pauseImmediately(); }
                }}
                onMouseUp={triggerResume}
                onTouchEnd={triggerResume}
                style={{ width: '100%' }}
              />
            </div>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: '12px', color: '#888' }}>#: <EditableValue {...evProps} fieldName="RCa" value={RCAbsolute} displayValue={RCAbsolute !== null ? RCAbsolute : 'N/A'} onCommit={v => { setRCAbsolute(Math.round(Math.max(0, Math.min(maxAbsoluteVal * 2, v)))); setRCPercent(null); pauseImmediately(); triggerResume(); }} /></div>
              <input type="range" min="0" max={maxAbsoluteVal * 2}
                value={RCAbsolute !== null ? RCAbsolute : 0}
                onChange={e => {
                  const v = Number(e.target.value);
                  if (v > 0 || RCPercent === null) { setRCAbsolute(v); setRCPercent(null); pauseImmediately(); }
                }}
                onMouseUp={triggerResume}
                onTouchEnd={triggerResume}
                style={{ width: '100%' }}
              />
            </div>
          </div>
        </div>
        
        {/* Cosmetic */}
        <h3 style={{ margin: '16px 0 12px 0', color: '#8cf' }}>Cosmetic</h3>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Base Hue: <EditableValue {...evProps} fieldName="hue" value={hue} displayValue={`${hue}°`} onCommit={v => { setHue(((v % 360) + 360) % 360); setRenderTrigger(r => r + 1); }} />
          <input type="range" min="0" max="360" value={hue}
            onChange={e => { setHue(Number(e.target.value)); setRenderTrigger(r => r + 1); }}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Rotation: <EditableValue {...evProps} fieldName="rotation" value={rotation} displayValue={`${rotation}°`} onCommit={v => setRotation(Math.max(-180, Math.min(180, v)))} />
          <input type="range" min="-180" max="180" value={rotation}
            onChange={e => setRotation(Number(e.target.value))}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Death Fade (FD): <EditableValue {...evProps} fieldName="FD" value={FD} displayValue={`${FD}%`} onCommit={v => setFD(Math.max(0, Math.min(100, Math.round(v))))} />
          <input type="range" min="0" max="100" value={FD}
            onChange={e => setFD(Number(e.target.value))}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Fade Proportional (FP): <EditableValue {...evProps} fieldName="FP" value={FP} onCommit={v => setFP(Math.max(2, Math.min(2048, Math.round(v))))} />
          <input type="range" min="2" max="2048" value={2050 - FP}
            onChange={e => setFP(2050 - Number(e.target.value))}
            style={{ width: '100%' }}
          />
        </label>
        
        <label style={{ display: 'block', marginBottom: '12px' }}>
          Fade Fixed (FF): <EditableValue {...evProps} fieldName="FF" value={FF} onCommit={v => setFF(Math.max(2, Math.min(2048, Math.round(v))))} />
          <input type="range" min="2" max="2048" value={2050 - FF}
            onChange={e => setFF(2050 - Number(e.target.value))}
            style={{ width: '100%' }}
          />
        </label>
      </div>
      
      {/* Tiled overlay - aligned with canvas */}
      {overlayOpen && (() => {
        // Calculate where canvas center is on screen (now at top with padding)
        const canvasCenterX = viewportWidth / 2;
        const canvasCenterY = CONTENT_PADDING + rotatedDisplayH / 2;
        
        // Tile dimensions (same scale as canvas core)
        const maxCoreDim = Math.max(W, H);
        let tileScale;
        if (maxCoreDim <= targetSize) {
          tileScale = Math.max(1, Math.floor(targetSize / maxCoreDim));
        } else {
          tileScale = targetSize / maxCoreDim;
        }
        const tileW = Math.round(W * tileScale);
        const tileH = Math.round(H * tileScale);
        
        // Background position: center the tile grid so core aligns with canvas
        // The canvas shows the core at its center, so we offset the background
        // so that a tile center lands at the canvas center
        const bgPosX = canvasCenterX - tileW / 2;
        const bgPosY = canvasCenterY - tileH / 2;
        
        return (
          <div
            onClick={handleOverlayClick}
            style={{
              position: 'fixed',
              top: rotation !== 0 ? '-50vh' : 0,
              left: rotation !== 0 ? '-50vw' : 0,
              width: rotation !== 0 ? '200vw' : '100vw',
              height: rotation !== 0 ? '200vh' : '100vh',
              backgroundColor: '#000',
              backgroundImage: tileUrl ? `url(${tileUrl})` : 'none',
              backgroundRepeat: 'repeat',
              backgroundPosition: `${bgPosX}px ${bgPosY}px`,
              backgroundSize: `${tileW}px ${tileH}px`,
              imageRendering: 'pixelated',
              cursor: 'pointer',
              zIndex: 9999,
              transform: rotation !== 0 ? `rotate(${rotation}deg)` : undefined,
              transformOrigin: rotation !== 0 ? '50vw 50vh' : undefined
            }}
          />
        );
      })()}
    </div>
  );
}
