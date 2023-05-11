import {Cell, Universe} from "game-of-life";
import {memory} from "game-of-life/game_of_life_bg.wasm";

const CELL_SIZE = 5; // px
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

// Construct universe and get dimensions
let universe = Universe.new();
const height = universe.height();
const width = universe.width();

// Give canvas room for all pixels wth a 1px border around each
const canvas = document.getElementById("game-of-life-canvas");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;

const ctx = canvas.getContext('2d');

let animationId = null;
let ticksPerFrame = 1;

const renderLoop = () => {
    fps.render();

    for (let i = 0; i < ticksPerFrame; i++) {
        universe.tick();
    }

    drawGrid();
    drawCells();
    animationId = requestAnimationFrame(renderLoop);
}

const isPaused = () => {
    return animationId === null;
}

/** Stuff associated with play/pause button **/
const playPauseButton = document.getElementById("play-pause");

const play = () => {
    playPauseButton.textContent = "⏸";
    renderLoop();
}

const pause = () => {
    playPauseButton.textContent = "▶";
    cancelAnimationFrame(animationId);
    animationId = null;
}

playPauseButton.addEventListener("click", event => {
    if (isPaused()) {
        play();
    } else {
        pause();
    }
});

/* Step stuff */
const step = () => {
    universe.tick();
    drawGrid();
    drawCells();
}
const stepButton = document.getElementById("step");

stepButton.addEventListener("click", event => {
    step();
})

/** Stuff associated with speed slider **/
const value = document.querySelector("#value");
const input = document.querySelector("#ticks-per-frame");

value.textContent = input.value
input.addEventListener("input", (event) => {
    value.textContent = event.target.value
    ticksPerFrame = event.target.value
})

/* Benchmark stuff */
const fps = new class {
    constructor() {
        this.fps = document.getElementById("fps");
        this.frames = [];
        this.lastFrameTimeStamp = performance.now();
    }

    render() {
        // Convert the delta time since the last frame render into a measure of frames per second.
        const now = performance.now();
        const delta = now - this.lastFrameTimeStamp;
        this.lastFrameTimeStamp = now;
        const fps = 1 / delta * 1000;

        // Save only the latest 100 timings
        this.frames.push(fps);
        if (this.frames.length > 100) {
            this.frames.shift();
        }

        // Find the max, min, and mean fo our 100 latest timings
        let max = -Infinity;
        let min = Infinity;
        let sum = 0;
        for (let i = 0; i < this.frames.length; i++) {
            sum += this.frames[i];
            min = Math.min(this.frames[i], min);
            max = Math.max(this.frames[i], max);
        }
        let mean = sum / this.frames.length;

        // Render the stats
        this.fps.textContent = `
        Frames per Second:
        latest = ${Math.round(fps)}
        avg of last 100 = ${Math.round(mean)}
        min of last 100 = ${Math.round(min)}
        max of last 100 = ${Math.round(max)}
        `.trim()
    }
};

/* Reset board to some random state */
const resetBoard = () => {
    universe = Universe.new();
}

const resetButton = document.getElementById("reset");

resetButton.addEventListener("click", event => {
    resetBoard();
})

/* Clear board so all cells are 'dead' */
const clearBoard = () => {
    universe = Universe.empty();
    drawGrid();
    drawCells();
}

const clearButton = document.getElementById("clear");

clearButton.addEventListener("click", event => {
    clearBoard();
})


const drawGrid = () => {
    ctx.beginPath();
    ctx.strokeStyle = GRID_COLOR;

    // Vertical lines
    for (let i = 0; i <= width; i++) {
        ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
        ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
    }

    // Horizontal lines
    for (let j = 0; j <= height; j++) {
        ctx.moveTo(0, j * (CELL_SIZE + 1) + 1);
        ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
    }

    ctx.stroke();
}

/* Draw some shapes */


const getIndex = (row, column) => {
    return row * width + column;
};

/*
* Takes an array of integers and checks if the nth bit is set.
* */
const bitIsSet = (n, arr) => {
    const byte = Math.floor(n / 8); // Get byte in array which contains bit in question
    const mask = 1 << (n % 8); // Shift `1` n bits to create mask at desired  bit
    return (arr[byte] & mask) === mask; // Bitwise AND of array and mask, returns true if bit is set
}

const drawCells = () => {
    const cellsPtr = universe.cells();
    const cells = new Uint8Array(memory.buffer, cellsPtr, width * height / 8);

    ctx.beginPath();

    // Set Alive cells
    ctx.fillStyle = ALIVE_COLOR;
    for (let row = 0; row < height; row++) {
        for (let col = 0; col < width; col++) {
            const idx = getIndex(row, col);
            if (!bitIsSet(idx, cells)) {
                continue;
            }

            ctx.fillRect(
                col * (CELL_SIZE + 1) + 1,
                row * (CELL_SIZE + 1) + 1,
                CELL_SIZE,
                CELL_SIZE
            );
        }
    }

    // Set Dead cells
    ctx.fillStyle = DEAD_COLOR;
    for (let row = 0; row < height; row++) {
        for (let col = 0; col < width; col++) {
            const idx = getIndex(row, col);
            if (bitIsSet(idx, cells)) {
                continue;
            }

            ctx.fillRect(
                col * (CELL_SIZE + 1) + 1,
                row * (CELL_SIZE + 1) + 1,
                CELL_SIZE,
                CELL_SIZE
            );
        }
    }

    ctx.stroke();
}

const validCells = (row, col, height, width, size) => {
    if (row < size || row > height - (size + 1) || col < size || col > width - (size + 1)) {
        console.log("WOULD OVERFLOW");
        return false;
    }
    return true;
}

canvas.addEventListener("click", event => {
    const boundingRect = canvas.getBoundingClientRect();

    const scaleX = canvas.width / boundingRect.width;
    const scaleY = canvas.height / boundingRect.height;

    const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
    const canvasTop = (event.clientY - boundingRect.top) * scaleY;

    const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
    const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

    if (event.shiftKey) {
        if (validCells(row, col, height, width, 6)) {
            console.log("NEED MOAR PULSARS");
            universe.spawn_pulsar(row, col);
        } else {
            console.log("WOULD OVERFLOW");
        }
    } else if (event.metaKey) {
        if (validCells(row, col, height, width, 2)) {
            console.log("LET THERE BE GLIDERS");
            universe.spawn_glider(row, col);
        } else {
            console.log("WOULD OVERFLOW");
        }
    } else {
        universe.toggle_cell(row, col);
    }

    drawGrid();
    drawCells();
})

drawGrid();
drawCells();
play();