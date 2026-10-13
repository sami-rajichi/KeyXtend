.pragma library
// The real pointer and the tests: a hand on the mouse during a run must not move a test's hover.

// Makes each window in `windows` click-through for the real pointer, setting its flags once as a value; the test's own
// mouse events still reach it.
function through(windows) {
    for (const w of windows)
        w.flags |= Qt.WindowTransparentForInput;
}
