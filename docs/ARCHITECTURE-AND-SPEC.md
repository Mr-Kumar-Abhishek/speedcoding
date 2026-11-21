# SpeedCo I Architecture & Technical Reference

Summary of the **IBM 701 Speedcoding System (SpeedCo I)** hardware/interpreter model encoded by the `speedcoding` crate, distilled from the June 1953 manual (`102632810-05-01-acc.pdf`).

---

## 1. Memory & 72-Bit Word Formats

### Electrostatic Memory Map (Pages 2-1, 3-2, 4-2, 9-11)
- **Total Electrostatic Capacity:** `1024` words (addresses `0000..=1023`).
- **Control Information Regions:** `0000..=0299` and `1014..=1023` (reserved for SpeedCo I control system; location `0025` stores `R`-quantity control information readable/modifiable only via specific `OP2` address-counter operations).
- **User Storage Region:** `0300..=1013` (**714 words**). Execution always begins at **`PC = 0300`**.
- **Auxiliary Storage:**
  - **Magnetic Drums `P` and `Q`:** `1024` words each (`0000..=1023`), no address wrap-around.
  - **Magnetic Tapes `J`, `K`, `L`, `M`:** Up to ~140,000 words per 1400-ft reel, variable-length unit records + checksum, 7 channels (6 data + 1 odd-parity redundancy channel).

### Floating Binary Number Word (Page 3-2)
Numbers inside the machine take the normalized form $F \cdot 2^E$, where $\frac{1}{2} \le |F| < 1$ or $F = 0$, and $|E| \le 131\,071$.

| 1-Based Bit Positions (MSB..LSB) | Width | Field |
|---|---:|---|
| `1` | 1 | Sign of quantity (`1` = negative) |
| `2–36` | 35 | Fractional part $F$ |
| `37` | 1 | Sign of binary exponent (`1` = negative) |
| `38–54` | 17 | Binary exponent $E$ (`0..=131_071`) |
| `55–72` | 18 | Unused (`0`) |

### External Floating Decimal (Pages 3-1, 7-4)
Numbers read from data cards or printed on output have the form $F \cdot 10^E$, where $F$ is a 10-digit decimal fraction ($\frac{1}{10} \le |F| < 1$ or $F = 0$) and $|E| \le 236$.

### Instruction Word Layout (Page 4-4)

| 1-Based Bit Positions | Width | Field | Notes |
|---|---:|---|---|
| `1` | 1 | Sign of instruction | Automatically supplied by SpeedCo I |
| `2` | 1 | `0` | Always zero |
| `3–5` | 3 | `R`-code | `0..=7`: bit 4 (`R_A`), bit 2 (`R_B`), bit 1 (`R_C`) |
| `6` | 1 | `0` | Always zero |
| `7–18` | 12 | `OP1` | 3-digit numerical operation code |
| `19–26` | 8 | `OP2` | 3-digit numerical operation code (`000` = blank/NoOp) |
| `27–36` | 10 | `D` | `OP2` address / operand |
| `37` | 1 | `0` | Always zero |
| `38–47` | 10 | `A` | First operand / start address |
| `48–57` | 10 | `B` | Second operand / end address |
| `58–59` | 2 | `00` | Always zero |
| `60–62` | 3 | `L`-code | `0` (switch 1), `3` (switch 2), `6` (switch 3) |
| `63–72` | 10 | `C` | Destination / drum / print ID address |

---

## 2. Arithmetic & Elementary Mathematical Rules (Pages 3-3, 3-4, Appendix 1–2)

- **Addition / Subtraction (`ADD`, `SUB`, `ADDAB`, `ABADD`, `SUBAB`, `ABSUB`):**
  - Operands whose decimal exponents differ by more than **76** cannot be processed (`ExponentDifferenceTooLarge`).
  - **Exact cancellation:** When operands cancel identically to zero, $F = 0$ is stored and the operand exponent is preserved without shifting.
  - Results returned to memory are **unrounded** (truncated to 35 bits).
- **Multiplication (`MPY`, `NGMPY`):** If $|F_A \cdot F_B| < \frac{1}{2}$, the fractional product is doubled and the sum of exponents is reduced by `1`. Unrounded.
- **Division (`DIV`, `NGDIV`):** Forms a **35-bit or 34-bit rounded** quotient; `F(B) == 0` triggers a **Divide-Check** stop.
- **Set 1 Elementary Functions (Appendix 1–2):**
  - `SQRT`: Newton iteration ($\ge 9$ sig. digits for $|E_{10}| \le 39\,456$; negative or zero $F$ yields $F=0$ with exponent $\lceil E_2 / 2 \rceil$).
  - `SINE`: Series approximation ($\ge 8$ decimal places for $|E_{10}| \le 3$; $F=0$ or $10 < n < 77$ yields $F=0, E_2 = +7$).
  - `ARTAN`: Principal value ($\ge 7$ decimal places for $|E_{10}| \le 76$).
  - `EXP`: Series approximation ($\ge 8$ sig. digits).
  - `LN`: Binary digit generation ($\ge 9$ decimal places for $1/e < x < e$, $\ge 9$ sig. digits otherwise; negative $F$ behaves as $+F$; $F=0$ behaves as binary $F = \frac{1}{2}$).

---

## 3. Complete Operation Code Reference (Section 5)

### `OP1` Operations — 49 Codes (Pages 5-1..5-2)

`*` marks skip-capable operations where **`OP2` must be left blank (`000`)**.

| Mnemonic | Code | Description | Mnemonic | Code | Description |
|---|---:|---|---|---:|---|
| `ADD` | 658 | $Q(A) + Q(B) \to Q(C)$ | `RBTPK`* | 417 | Read backward tape K |
| `SUB` | 696 | $Q(A) - Q(B) \to Q(C)$ | `RBTPL`* | 418 | Read backward tape L |
| `ADDAB` | 699 | $Q(A) + \|Q(B)\| \to Q(C)$ | `RBTPM`* | 419 | Read backward tape M |
| `ABADD` | 703 | $\|Q(A)\| + \|Q(B)\| \to Q(C)$ | `SFTPJ`* | 556 | Skip forward tape J |
| `SUBAB` | 707 | $Q(A) - \|Q(B)\| \to Q(C)$ | `SFTPK`* | 557 | Skip forward tape K |
| `ABSUB` | 711 | $\|Q(A)\| - \|Q(B)\| \to Q(C)$ | `SFTPL`* | 558 | Skip forward tape L |
| `MPY` | 715 | $Q(A) \cdot Q(B) \to Q(C)$ | `SFTPM`* | 559 | Skip forward tape M |
| `NGMPY` | 731 | $-(Q(A) \cdot Q(B)) \to Q(C)$ | `SBTPJ`* | 546 | Skip backward tape J |
| `DIV` | 734 | $Q(A) \div Q(B) \to Q(C)$ | `SBTPK`* | 547 | Skip backward tape K |
| `NGDIV` | 748 | $-(Q(A) \div Q(B)) \to Q(C)$ | `SBTPL`* | 548 | Skip backward tape L |
| `SQRT` | 782 | $\sqrt{Q(A)} \to Q(C)$ | `SBTPM`* | 549 | Skip backward tape M |
| `SINE` | 780 | $\sin(Q(A)) \to Q(C)$ | `RWTPJ` | 572 | Rewind tape J |
| `ARTAN` | 781 | $\arctan(Q(A)) \to Q(C)$ | `RWTPK` | 574 | Rewind tape K |
| `EXP` | 783 | $e^{Q(A)} \to Q(C)$ | `RWTPL` | 576 | Rewind tape L |
| `LN` | 784 | $\ln(Q(A)) \to Q(C)$ | `RWTPM` | 578 | Rewind tape M |
| `WRTPJ` | 532 | Write cells $A..B$ on tape J | `EFTPJ` | 564 | End file tape J (6" erase) |
| `WRTPK` | 533 | Write cells $A..B$ on tape K | `EFTPK` | 566 | End file tape K (6" erase) |
| `WRTPL` | 534 | Write cells $A..B$ on tape L | `EFTPL` | 568 | End file tape L (6" erase) |
| `WRTPM` | 535 | Write cells $A..B$ on tape M | `EFTPM` | 570 | End file tape M (6" erase) |
| `RFTPJ`* | 435 | Read forward tape J | `WRDRP`* | 497 | Write cells $A..B$ to drum P at $C$ |
| `RFTPK`* | 437 | Read forward tape K | `WRDRQ`* | 498 | Write cells $A..B$ to drum Q at $C$ |
| `RFTPL`* | 439 | Read forward tape L | `RFDRP` | 526 | Read drum P at $C$ to $A..B$ |
| `RFTPM`* | 441 | Read forward tape M | `RFDRQ` | 528 | Read drum Q at $C$ to $A..B$ |
| `RBTPJ`* | 416 | Read backward tape J | `PRINT` | 580 | Print $A..B$ (5/line, ID `100*C + n`) |
| | | | `NOOP` | 751 | No `OP1` operation (`2.736 ms`) |

### `OP2` Operations — 48 Codes (Pages 5-3..5-5)

| Mnemonic | Code | Description | Mnemonic | Code | Description |
|---|---:|---|---|---:|---|
| `TR` | 104 | Transfer unconditionally to $D$ | `SKRC` | 162 | Skip next if $R_C = D$ |
| `TRPL` | 109 | Transfer to $D$ if $Q(C) > 0$ | `RADDA` | 199 | Reset AC and add $A$ of cell $D$ |
| `TRMN` | 115 | Transfer to $D$ if $Q(C) < 0$ | `RADDB` | 202 | Reset AC and add $B$ of cell $D$ |
| `TRZ` | 112 | Transfer to $D$ if $F(Q(C)) = 0$ | `RADDC` | 205 | Reset AC and add $C$ of cell $D$ |
| `SNTRP` | 117 | Transfer to $D$ if switch 4 (`P`) down | `RADDD` | 208 | Reset AC and add $D$ of cell $D$ |
| `SNTRQ` | 120 | Transfer to $D$ if switch 5 (`Q`) down | `ADDA` | 177 | Add $A$ of cell $D$ to AC |
| `TIA` | 128 | $R_A + 1 \to R_A$, transfer to $D$ | `ADDB` | 184 | Add $B$ of cell $D$ to AC |
| `TIB` | 126 | $R_B + 1 \to R_B$, transfer to $D$ | `ADDC` | 190 | Add $C$ of cell $D$ to AC |
| `TIC` | 125 | $R_C + 1 \to R_C$, transfer to $D$ | `ADDD` | 193 | Add $D$ of cell $D$ to AC |
| `TIAB` | 130 | Increase $R_A, R_B$, transfer to $D$ | `SUBA` | 211 | Subtract $A$ of cell $D$ from AC |
| `TIBC` | 127 | Increase $R_B, R_C$, transfer to $D$ | `SUBB` | 216 | Subtract $B$ of cell $D$ from AC |
| `TIAC` | 129 | Increase $R_A, R_C$, transfer to $D$ | `SUBC` | 221 | Subtract $C$ of cell $D$ from AC |
| `TIABC` | 131 | Increase $R_A, R_B, R_C$, transfer to $D$ | `SUBD` | 226 | Subtract $D$ of cell $D$ from AC |
| `TDA` | 135 | Decrease $R_A$, transfer to $D$ | `STA` | 251 | Store $\|\text{AC}\| \bmod 1024$ into $A$ of $D$ |
| `TDB` | 133 | Decrease $R_B$, transfer to $D$ | `STB` | 252 | Store $\|\text{AC}\| \bmod 1024$ into $B$ of $D$ |
| `TDC` | 132 | Decrease $R_C$, transfer to $D$ | `STC` | 235 | Store $\|\text{AC}\| \bmod 1024$ into $C$ of $D$ |
| `TDAB` | 137 | Decrease $R_A, R_B$, transfer to $D$ | `STD` | 244 | Store $\|\text{AC}\| \bmod 1024$ into $D$ of $D$ |
| `TDBC` | 134 | Decrease $R_B, R_C$, transfer to $D$ | `SKIP` | 165 | Skip next if $\|\text{AC}\| = D$ |
| `TDAC` | 136 | Decrease $R_A, R_C$, transfer to $D$ | `PRCH` | 232 | Prepare checking loop |
| `TDABC` | 138 | Decrease $R_A, R_B, R_C$, transfer to $D$ | `STCH` | 253 | Start checking loop |
| `SETRA` | 139 | $D \to R_A$ | `ECHTR` | 254 | End check & transfer (`D` / compare) |
| `SETRB` | 250 | $D \to R_B$ | `STOP` | 123 | Stop machine; restart transfers to $D$ |
| `SETRC` | 145 | $D \to R_C$ | *(blank)* | 000 | No `OP2` operation (`0.000 ms`) |
| `SKRA` | 152 | Skip next if $R_A = D$ | | | |
| `SKRB` | 159 | Skip next if $R_B = D$ | | | |

---

## 4. Punched Cards, Tapes, Checking, Listing & Timing

### 80-Column Card Dispatch (Columns 8–9)
- **Instruction Card (Pages 7-1..7-3):** Col `8` never `0+1`, Col `9` blank, `10–13` Location, `14` `0`, `15–19` Alpha `OP1`, `20` `R`, `21–24` `A`, `25–28` `B`, `29–32` `C`, `33–34` `00`, `35–39` Alpha `OP2`, `40–43` `D`, `44` `L`, `46–48` Num `OP1`, `49–51` Num `OP2`, `52–55` `0000`.
- **Data Card (Pages 7-3..7-4):** Col `8` double-punched `0+1`, Col `9` = `8`, `10–13` First Location, up to 5 floating decimals (`N1..N5` + `n1..n5` with **Col 45 skipped** inside `N3` and `12`/`11` overpunched signs), Col `80` word count (`1..=5`).
- **Loading Control Card (Page 7-17):** Col `8` not `0+1`, Col `9` = `9`, `10` = `0`, `11–14` first addr, `15` = `0`, `16–19` last addr, `20` = `0`, `21–24` drum addr (`0000` for tape), `25–28` = `0000`, `29` unit (`J..M` or `P`/`Q`).

### Tape Status Transition Matrix (Pages 7-13..7-14)

| Instruction | Original `Read` | Original `Neutral` | Original `Write` |
|---|---|---|---|
| `RFTP` / `RBTP` / `SFTP` / `SBTP` | `Read` | `Read` | `Write` (Copy-check stop) |
| `WRTP` | `Read` (Copy-check stop) | `Write` | `Write` |
| `EFTP` | `Read` (Ignored, no stop) | `Neutral` | `Neutral` |
| `RWTP` | `Neutral` | `Neutral` | `Neutral` |

### Checking, Listing & Timing Highlights (Sections 10–12)
- **Checking (`PRCH`/`STCH`/`ECHTR`):** Pass 1 accumulates fixed-point sums into Check Counter 1 and loops to `D`; Pass 2 accumulates into Check Counter 2 and skips the instruction after `ECHTR` if counters match. Calculations between `ECHTR` and `STCH` are excluded; `PRINT` is suppressed on Pass 2 (`2.896 ms`).
- **Selective Listing (`L`-code `0`, `3`, `6`):** Records post-`OP1` / pre-`OP2` state into a drum buffer flushed every **10 lines** (`1.325 s` avg / `1.370 s` max per line). Exponents with $|E_{10}| > 236$ print `0` fraction and `±999` exponent; `SUB`, `ABADD`, `ABSUB`, `ADDAB`, `SUBAB`, `NGMPY`, and `NGDIV` reflect sign transformations on listed operands.
- **Tape & Loading Timing (Pages 12-3, 12-5):**
  - Tape length (inches): $2 \cdot 72 + \sum (n_i + 0.5) \cdot \frac{3}{25} + (k - 1) \cdot 1$, rewind at $75\text{ in/s}$, EOF gap $72/75 = 0.96\text{ s}$.
  - Card loading time (seconds): $0.4n + 0.6c + e$ ($e = 2$ if SpeedCo I control deck is already on the two drums, else $e = 55$).
