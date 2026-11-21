(function () {
  "use strict";

  const THEME_KEY = "speedcoding-theme";
  const root = document.documentElement;
  const themeBtn = document.getElementById("theme-toggle");

  function currentTheme() {
    return root.getAttribute("data-theme") === "light" ? "light" : "dark";
  }

  function applyTheme(theme, persist) {
    const t = theme === "light" ? "light" : "dark";
    root.setAttribute("data-theme", t);
    if (themeBtn) {
      themeBtn.textContent = t === "dark" ? "Light Mode" : "Dark Mode";
      themeBtn.setAttribute("aria-pressed", t === "light" ? "true" : "false");
    }
    if (persist) {
      try { localStorage.setItem(THEME_KEY, t); } catch (_) {}
    }
    window.dispatchEvent(new CustomEvent("speedcoding:themechange", { detail: { theme: t } }));
  }

  applyTheme(currentTheme(), false);

  if (themeBtn) {
    themeBtn.addEventListener("click", () => {
      applyTheme(currentTheme() === "dark" ? "light" : "dark", true);
    });
  }

  // Reading progress bar + Back to top
  const progressBar = document.getElementById("read-progress");
  const topBtn = document.getElementById("back-to-top");

  function updateScrollUI() {
    const doc = document.documentElement;
    const maxScroll = Math.max(1, doc.scrollHeight - doc.clientHeight);
    if (progressBar) {
      const pct = Math.min(100, Math.max(0, (window.scrollY / maxScroll) * 100));
      progressBar.style.width = pct.toFixed(1) + "%";
    }
    if (topBtn) {
      topBtn.classList.toggle("visible", window.scrollY > 420);
    }
  }

  window.addEventListener("scroll", updateScrollUI, { passive: true });
  updateScrollUI();

  if (topBtn) {
    topBtn.addEventListener("click", () => {
      window.scrollTo({ top: 0, behavior: "smooth" });
    });
  }

  // Clipboard helper with fallback
  function copyText(text, onDone) {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(onDone, () => fallbackCopy(text, onDone));
    } else {
      fallbackCopy(text, onDone);
    }
  }

  function fallbackCopy(text, onDone) {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.setAttribute("readonly", "");
    ta.style.position = "fixed";
    ta.style.opacity = "0";
    document.body.appendChild(ta);
    ta.select();
    try { document.execCommand("copy"); } catch (_) {}
    document.body.removeChild(ta);
    onDone();
  }

  // Auto-wrap <pre> blocks with Copy button
  document.querySelectorAll("pre").forEach((pre) => {
    if (pre.parentElement && pre.parentElement.classList.contains("code-wrap")) return;
    const wrap = document.createElement("div");
    wrap.className = "code-wrap";
    pre.parentNode.insertBefore(wrap, pre);
    wrap.appendChild(pre);

    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "copy-btn";
    btn.textContent = "Copy";
    btn.setAttribute("aria-label", "Copy code block");
    btn.addEventListener("click", () => {
      copyText(pre.textContent || "", () => {
        btn.textContent = "Copied!";
        btn.classList.add("copied");
        setTimeout(() => {
          btn.textContent = "Copy";
          btn.classList.remove("copied");
        }, 1600);
      });
    });
    wrap.appendChild(btn);
  });

  // One-click SHA-256 copy buttons
  document.querySelectorAll(".sha256-copy[data-copy]").forEach((btn) => {
    const tag = btn.querySelector(".sha-tag");
    const originalTag = tag ? tag.textContent : "Copy";
    btn.addEventListener("click", () => {
      const value = btn.getAttribute("data-copy") || "";
      copyText(value, () => {
        btn.classList.add("copied");
        if (tag) tag.textContent = "Copied!";
        setTimeout(() => {
          btn.classList.remove("copied");
          if (tag) tag.textContent = originalTag;
        }, 1600);
      });
    });
  });

  // Print report button
  const printBtn = document.getElementById("print-report");
  if (printBtn) {
    printBtn.addEventListener("click", () => window.print());
  }

  // Sidebar TOC Scroll-Spy (docs/index.html)
  const tocLinks = Array.from(document.querySelectorAll(".toc-sidebar .toc-link[href^='#']"));
  const docSections = Array.from(document.querySelectorAll("section.doc-section[id]"));
  if (tocLinks.length && docSections.length && "IntersectionObserver" in window) {
    const linkById = {};
    tocLinks.forEach((a) => {
      const id = a.getAttribute("href").slice(1);
      linkById[id] = a;
    });

    function setActiveToc(id) {
      tocLinks.forEach((a) => a.classList.remove("active"));
      if (linkById[id]) linkById[id].classList.add("active");
    }

    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((e) => e.isIntersecting)
          .sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top);
        if (visible.length) setActiveToc(visible[0].target.id);
      },
      { rootMargin: "-90px 0px -65% 0px", threshold: 0.01 }
    );

    docSections.forEach((sec) => observer.observe(sec));
  }

  // Interactive 72-bit Word & Instruction Visualizer
  const BIT_FIELDS = {
    "float-sign": {
      title: "Quantity Sign Bit",
      range: "Bit 1 (MSB)",
      width: "1 bit",
      domain: "0 = positive (+), 1 = negative (-)",
      ref: "Manual Sec. 3-2 (Floating Binary Number Word)",
      notes: "Signed-magnitude representation; applies independently from exponent sign at Bit 37."
    },
    "float-frac": {
      title: "Normalized Fraction F",
      range: "Bits 2–36",
      width: "35 bits",
      domain: "0.5 <= |F| < 1 (normalized) or 0",
      ref: "Manual Sec. 3-1..3-2",
      notes: "35-bit binary fraction held unrounded in OP1 add/sub/multiply; DIV/NGDIV form 34/35-bit rounded quotients."
    },
    "float-esign": {
      title: "Binary Exponent Sign",
      range: "Bit 37",
      width: "1 bit",
      domain: "0 = positive exponent, 1 = negative exponent",
      ref: "Manual Sec. 3-2",
      notes: "Signs the 17-bit binary exponent magnitude in Bits 38–54."
    },
    "float-exp": {
      title: "Binary Exponent Magnitude E",
      range: "Bits 38–54",
      width: "17 bits",
      domain: "|E2| <= 131,071 (decimal conversion |E10| <= 236)",
      ref: "Manual Sec. 3-1..3-2",
      notes: "Exceeding |E10| > 236 formats as +/-999 on selective listing; decimal exponent diff > 76 in add/sub is rejected."
    },
    "float-unused": {
      title: "Unused / Reserved Half-Word Tail",
      range: "Bits 55–72 (LSB)",
      width: "18 bits",
      domain: "Must remain 0",
      ref: "Manual Sec. 3-2",
      notes: "SpeedCo I packs floating-binary numbers cleanly in Bits 1–54 of the 72-bit IBM 701 double-word."
    },
    "instr-sign": {
      title: "Instruction Sign Bit",
      range: "Bit 1",
      width: "1 bit",
      domain: "Sign bit of 72-bit instruction word",
      ref: "Manual Sec. 4-4",
      notes: "Bit 1 of the first 36-bit full word; follows IBM 701 signed full-word layout."
    },
    "instr-pad1": {
      title: "Mandatory Zero Bit",
      range: "Bit 2",
      width: "1 bit",
      domain: "Always 0",
      ref: "Manual Sec. 4-4",
      notes: "Must be zero in valid Speedcoding 72-bit instruction words."
    },
    "instr-r": {
      title: "R-Code (Index Register Selector)",
      range: "Bits 3–5",
      width: "3 bits",
      domain: "0..=7 (A=1, B=2, C=4 flags)",
      ref: "Manual Sec. 4-4, 9-1..9-6",
      notes: "Selects non-destructive address modification by R_A, R_B, and/or R_C on effective A, B, and C addresses."
    },
    "instr-pad2": {
      title: "Mandatory Zero Bit",
      range: "Bit 6",
      width: "1 bit",
      domain: "Always 0",
      ref: "Manual Sec. 4-4",
      notes: "Reserved separator between R-code and 12-bit OP1 code."
    },
    "instr-op1": {
      title: "OP1 Numeric Code",
      range: "Bits 7–18",
      width: "12 bits",
      domain: "49 OP1 operations (e.g. 658 ADD .. 784 LN)",
      ref: "Manual Sec. 4-4, 5-1..5-2",
      notes: "Executes before OP2; skip-capable OP1 codes (RFTP*, RBTP*, SFTP*, SBTP*, WRDRP/Q) forbid a non-zero OP2."
    },
    "instr-op2": {
      title: "OP2 Numeric Code",
      range: "Bits 19–26",
      width: "8 bits",
      domain: "48 OP2 operations (000 No-Op .. 254 ECHTR)",
      ref: "Manual Sec. 4-4, 5-3..5-5",
      notes: "Transfers, sense switches, R-register operations, Address Counter ops, PRCH/STCH/ECHTR checking, or STOP."
    },
    "instr-d": {
      title: "D Address Field",
      range: "Bits 27–36",
      width: "10 bits",
      domain: "0000..=1023",
      ref: "Manual Sec. 4-4, 5-3..5-5",
      notes: "Target address for OP2 transfers, R-register immediate loads/comparisons, or Address Counter cell operations."
    },
    "instr-pad3": {
      title: "Mandatory Zero Bit",
      range: "Bit 37",
      width: "1 bit",
      domain: "Always 0",
      ref: "Manual Sec. 4-4",
      notes: "First bit of second 36-bit word in instruction layout; fixed zero."
    },
    "instr-a": {
      title: "A Operand Address",
      range: "Bits 38–47",
      width: "10 bits",
      domain: "0000..=1023 (modified by R_A when R bit A set)",
      ref: "Manual Sec. 4-4, 9-2",
      notes: "First operand cell Q(A) for OP1 arithmetic/math or block start cell A..B for tape/drum/print I/O."
    },
    "instr-b": {
      title: "B Operand Address",
      range: "Bits 48–57",
      width: "10 bits",
      domain: "0000..=1023 (modified by R_B when R bit B set)",
      ref: "Manual Sec. 4-4, 9-2",
      notes: "Second operand cell Q(B) for arithmetic or block end cell for tape/drum/print I/O."
    },
    "instr-pad4": {
      title: "Mandatory Zero Bits",
      range: "Bits 58–59",
      width: "2 bits",
      domain: "Always 00",
      ref: "Manual Sec. 4-4",
      notes: "Corresponds to card columns 33–34 fixed zeros on 80-column SpeedCo I instruction cards."
    },
    "instr-l": {
      title: "L-Code (Selective Listing Selector)",
      range: "Bits 60–62",
      width: "3 bits",
      domain: "0, 3, or 6 (Console switches 1, 2, 3)",
      ref: "Manual Sec. 4-4, 11-1..11-2",
      notes: "Controls selective diagnostic capture post-OP1 / pre-OP2; buffered in 10-line batches to drum/printer."
    },
    "instr-c": {
      title: "C Result / Auxiliary Address",
      range: "Bits 63–72 (LSB)",
      width: "10 bits",
      domain: "0000..=1023 (modified by R_C when R bit C set)",
      ref: "Manual Sec. 4-4, 9-2",
      notes: "Destination cell Q(C) for arithmetic/math, drum start address, print block ID 100*C+n, or TRPL/TRMN/TRZ test cell."
    }
  };

  document.querySelectorAll(".bit-visualizer").forEach((viz) => {
    const segs = Array.from(viz.querySelectorAll(".bit-seg[data-field]"));
    const inspector = viz.querySelector(".bit-inspector");
    if (!segs.length || !inspector) return;

    function selectSeg(seg) {
      const info = BIT_FIELDS[seg.getAttribute("data-field")];
      if (!info) return;
      segs.forEach((s) => s.classList.toggle("active", s === seg));
      inspector.innerHTML =
        `<div class="bit-inspector-title">${info.title} (${info.range})</div>` +
        `<div class="bit-inspector-meta">Width: <strong>${info.width}</strong> &middot; Domain: <code>${info.domain}</code> &middot; ${info.ref}</div>` +
        `<div>${info.notes}</div>`;
    }

    segs.forEach((seg) => {
      seg.addEventListener("click", () => selectSeg(seg));
    });
  });

  // Enhanced Opcode Explorer (#opcodes)
  const opFilterInput = document.getElementById("opcode-filter");
  const opCountLabel = document.getElementById("opcode-count");
  const opGrid = document.querySelector(".opcodes-grid");
  const opChips = Array.from(document.querySelectorAll("#opcodes .chip[data-cat]"));
  const opViewBtns = Array.from(document.querySelectorAll("#opcodes [data-opcode-view]"));
  const opRows = Array.from(document.querySelectorAll("#op1-table tbody tr, #op2-table tbody tr"));

  if (opFilterInput && opRows.length) {
    let activeCat = "all";
    let activeView = "both";

    function updateOpcodes() {
      const q = opFilterInput.value.trim().toLowerCase();
      let shown = 0;
      opRows.forEach((tr) => {
        const tableVisible =
          activeView === "both" ||
          (activeView === "op1" && tr.closest("#op1-table")) ||
          (activeView === "op2" && tr.closest("#op2-table"));
        const cats = (tr.getAttribute("data-cat") || "").split(/\s+/);
        const catMatch = activeCat === "all" || cats.includes(activeCat);
        const textMatch = !q || tr.textContent.toLowerCase().includes(q);
        const show = tableVisible && catMatch && textMatch;
        tr.style.display = show ? "" : "none";
        if (show) shown += 1;
      });

      if (opCountLabel) {
        if (!q && activeCat === "all" && activeView === "both") {
          opCountLabel.textContent = `Showing all 97 opcodes (${opRows.length} rows)`;
        } else {
          opCountLabel.textContent = `Showing ${shown} of ${opRows.length} rows`;
        }
      }
    }

    opFilterInput.addEventListener("input", updateOpcodes);

    opChips.forEach((chip) => {
      chip.addEventListener("click", () => {
        activeCat = chip.getAttribute("data-cat") || "all";
        opChips.forEach((c) => c.classList.toggle("active", c === chip));
        updateOpcodes();
      });
    });

    opViewBtns.forEach((btn) => {
      btn.addEventListener("click", () => {
        activeView = btn.getAttribute("data-opcode-view") || "both";
        opViewBtns.forEach((b) => b.classList.toggle("active", b === btn));
        if (opGrid) {
          opGrid.classList.remove("view-op1", "view-op2");
          if (activeView === "op1") opGrid.classList.add("view-op1");
          if (activeView === "op2") opGrid.classList.add("view-op2");
        }
        updateOpcodes();
      });
    });
  }

  // TDD Accordion Search & Expand/Collapse (#tdd)
  const tddSection = document.getElementById("tdd");
  const tddFilter = document.getElementById("tdd-filter");
  const tddCount = document.getElementById("tdd-count");
  const tddExpandAll = document.getElementById("tdd-expand-all");
  const tddCollapseAll = document.getElementById("tdd-collapse-all");

  if (tddSection) {
    const tddDetails = Array.from(tddSection.querySelectorAll("details"));
    const tddItems = Array.from(tddSection.querySelectorAll("details li"));

    if (tddFilter) {
      tddFilter.addEventListener("input", () => {
        const q = tddFilter.value.trim().toLowerCase();
        let shown = 0;
        tddDetails.forEach((det) => {
          const items = Array.from(det.querySelectorAll("li"));
          let detMatches = 0;
          items.forEach((li) => {
            const match = !q || li.textContent.toLowerCase().includes(q);
            li.style.display = match ? "" : "none";
            if (match) {
              detMatches += 1;
              shown += 1;
            }
          });
          det.style.display = !q || detMatches > 0 ? "" : "none";
          if (q && detMatches > 0) det.open = true;
        });
        if (tddCount) {
          tddCount.textContent = q ? `Showing ${shown} of ${tddItems.length} phase tests` : `All ${tddItems.length} phase tests`;
        }
      });
    }

    if (tddExpandAll) {
      tddExpandAll.addEventListener("click", () => {
        tddDetails.forEach((d) => { d.open = true; });
      });
    }
    if (tddCollapseAll) {
      tddCollapseAll.addEventListener("click", () => {
        tddDetails.forEach((d) => { d.open = false; });
      });
    }
  }

  // Reports Portal Tabs (#tab-benchmark / #tab-compliance)
  const tabBenchmark = document.getElementById("tab-benchmark");
  const tabCompliance = document.getElementById("tab-compliance");
  const panelBenchmark = document.getElementById("panel-benchmark");
  const panelCompliance = document.getElementById("panel-compliance");

  if (tabBenchmark && tabCompliance && panelBenchmark && panelCompliance) {
    const tabs = {
      benchmark: { btn: tabBenchmark, panel: panelBenchmark },
      compliance: { btn: tabCompliance, panel: panelCompliance }
    };

    function selectTab(name, updateUrl) {
      const key = tabs[name] ? name : "benchmark";
      Object.keys(tabs).forEach((k) => {
        const active = k === key;
        tabs[k].btn.classList.toggle("active", active);
        tabs[k].btn.setAttribute("aria-selected", active ? "true" : "false");
        tabs[k].panel.classList.toggle("active", active);
      });
      if (updateUrl && window.history && window.history.replaceState) {
        window.history.replaceState(null, "", "#" + key);
      }
      updateScrollUI();
    }

    tabBenchmark.addEventListener("click", () => selectTab("benchmark", true));
    tabCompliance.addEventListener("click", () => selectTab("compliance", true));

    [tabBenchmark, tabCompliance].forEach((btn) => {
      btn.addEventListener("keydown", (e) => {
        if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
          const next = btn === tabBenchmark ? "compliance" : "benchmark";
          selectTab(next, true);
          tabs[next].btn.focus();
        }
      });
    });

    const params = new URLSearchParams(window.location.search);
    const queryTab = (params.get("tab") || "").toLowerCase();
    const hashTab = (window.location.hash || "").replace("#", "").toLowerCase();
    if (tabs[hashTab]) selectTab(hashTab, false);
    else if (tabs[queryTab]) selectTab(queryTab, false);

    window.addEventListener("hashchange", () => {
      const h = (window.location.hash || "").replace("#", "").toLowerCase();
      if (tabs[h]) selectTab(h, false);
    });
  }

  // Compliance Traceability Table Filter (report/index.html)
  const traceFilter = document.getElementById("trace-filter");
  const traceRows = Array.from(document.querySelectorAll("#trace-table tbody tr"));
  const traceChips = Array.from(document.querySelectorAll(".trace-chip[data-status]"));
  const traceCount = document.getElementById("trace-count");

  if (traceFilter && traceRows.length) {
    let traceStatus = "all";

    function updateTrace() {
      const q = traceFilter.value.trim().toLowerCase();
      let shown = 0;
      traceRows.forEach((tr) => {
        const st = tr.getAttribute("data-status") || "";
        const stOk = traceStatus === "all" || st === traceStatus;
        const txtOk = !q || tr.textContent.toLowerCase().includes(q);
        const show = stOk && txtOk;
        tr.style.display = show ? "" : "none";
        if (show) shown += 1;
      });
      if (traceCount) {
        traceCount.textContent = !q && traceStatus === "all"
          ? `Showing all ${traceRows.length} targets (71 tests)`
          : `Showing ${shown} of ${traceRows.length} targets`;
      }
    }

    traceFilter.addEventListener("input", updateTrace);
    traceChips.forEach((chip) => {
      chip.addEventListener("click", () => {
        traceStatus = chip.getAttribute("data-status") || "all";
        traceChips.forEach((c) => c.classList.toggle("active", c === chip));
        updateTrace();
      });
    });
  }

  // Global Quick-Jump Search Modal (docs/index.html)
  const searchModal = document.getElementById("search-modal");
  const searchInput = document.getElementById("search-input");
  const searchResults = document.getElementById("search-results");
  const searchEmpty = document.getElementById("search-empty");
  const openSearchBtn = document.getElementById("open-search");

  if (searchModal && searchInput && searchResults) {
    const items = [];

    document.querySelectorAll("section.doc-section[id]").forEach((sec) => {
      const h2 = sec.querySelector("h2");
      if (!h2) return;
      items.push({
        cat: "Section",
        title: h2.textContent.trim(),
        sub: "#" + sec.id,
        el: sec
      });
    });

    document.querySelectorAll("#op1-table tbody tr").forEach((tr) => {
      const cells = tr.querySelectorAll("td");
      if (cells.length < 4) return;
      items.push({
        cat: "OP1",
        title: `${cells[0].textContent.trim()} (${cells[1].textContent.trim()})`,
        sub: cells[3].textContent.trim(),
        el: tr
      });
    });

    document.querySelectorAll("#op2-table tbody tr").forEach((tr) => {
      const cells = tr.querySelectorAll("td");
      if (cells.length < 3) return;
      items.push({
        cat: "OP2",
        title: `${cells[0].textContent.trim()} (${cells[1].textContent.trim()})`,
        sub: cells[2].textContent.trim(),
        el: tr
      });
    });

    document.querySelectorAll("#backlog-table tbody tr").forEach((tr) => {
      const cells = tr.querySelectorAll("td");
      if (cells.length < 3) return;
      items.push({
        cat: "Backlog",
        title: `${cells[0].textContent.trim()} — ${cells[1].textContent.trim()}`,
        sub: cells[2].textContent.trim(),
        el: tr
      });
    });

    document.querySelectorAll("#tdd details li").forEach((li) => {
      const code = li.querySelector("code");
      const title = code ? code.textContent.trim() : li.textContent.trim();
      items.push({
        cat: "TDD Test",
        title,
        sub: li.textContent.trim(),
        el: li
      });
    });

    let selectedIndex = 0;
    let currentMatches = [];

    function flashTarget(el) {
      const parentDet = el.closest("details");
      if (parentDet) parentDet.open = true;
      if (el.closest("#op1-table, #op2-table")) {
        el.style.display = "";
      }
      el.scrollIntoView({ behavior: "smooth", block: "center" });
      el.classList.remove("row-flash");
      void el.offsetWidth;
      el.classList.add("row-flash");
    }

    function openSearch() {
      searchModal.hidden = false;
      searchInput.value = "";
      renderResults("");
      setTimeout(() => searchInput.focus(), 10);
    }

    function closeSearch() {
      searchModal.hidden = true;
    }

    function renderResults(rawQ) {
      const q = rawQ.trim().toLowerCase();
      currentMatches = (q
        ? items.filter((it) =>
            it.title.toLowerCase().includes(q) ||
            it.sub.toLowerCase().includes(q) ||
            it.cat.toLowerCase().includes(q)
          )
        : items.filter((it) => it.cat === "Section")
      ).slice(0, 18);

      selectedIndex = 0;
      searchResults.innerHTML = "";
      if (searchEmpty) searchEmpty.hidden = currentMatches.length > 0;

      currentMatches.forEach((it, idx) => {
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = "search-item" + (idx === 0 ? " selected" : "");
        btn.innerHTML =
          `<div><div class="search-item-title">${it.title}</div>` +
          `<div class="search-item-sub">${it.sub}</div></div>` +
          `<span class="search-cat">${it.cat}</span>`;
        btn.addEventListener("click", () => {
          closeSearch();
          flashTarget(it.el);
        });
        searchResults.appendChild(btn);
      });
    }

    function updateSelected(nextIdx) {
      const btns = Array.from(searchResults.querySelectorAll(".search-item"));
      if (!btns.length) return;
      selectedIndex = (nextIdx + btns.length) % btns.length;
      btns.forEach((b, i) => b.classList.toggle("selected", i === selectedIndex));
      btns[selectedIndex].scrollIntoView({ block: "nearest" });
    }

    if (openSearchBtn) openSearchBtn.addEventListener("click", openSearch);

    searchInput.addEventListener("input", () => renderResults(searchInput.value));

    searchInput.addEventListener("keydown", (e) => {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        updateSelected(selectedIndex + 1);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        updateSelected(selectedIndex - 1);
      } else if (e.key === "Enter" && currentMatches[selectedIndex]) {
        e.preventDefault();
        const chosen = currentMatches[selectedIndex];
        closeSearch();
        flashTarget(chosen.el);
      } else if (e.key === "Escape") {
        e.preventDefault();
        closeSearch();
      }
    });

    searchModal.addEventListener("click", (e) => {
      if (e.target === searchModal) closeSearch();
    });

    document.addEventListener("keydown", (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        if (searchModal.hidden) openSearch();
        else closeSearch();
      } else if (
        e.key === "/" &&
        searchModal.hidden &&
        !["INPUT", "TEXTAREA", "SELECT"].includes((document.activeElement || {}).tagName)
      ) {
        e.preventDefault();
        openSearch();
      } else if (e.key === "Escape" && !searchModal.hidden) {
        closeSearch();
      }
    });
  }
})();
