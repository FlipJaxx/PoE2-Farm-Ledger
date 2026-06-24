// Shared value formatting. Exalted Orb is the base unit (1 ex).
// Large amounts are displayed in Divine Orb to keep numbers readable.

let divineRate = 120;

export function setDivineRate(rate: number): void {
  if (Number.isFinite(rate) && rate > 0) divineRate = rate;
}

export function getDivineRate(): number {
  return divineRate;
}

export function fmtNumber(value: number, digits = 1): string {
  return Number.isFinite(value)
    ? value.toLocaleString(undefined, { maximumFractionDigits: digits })
    : '0';
}

export function fmtValue(exalts: number, rate = divineRate): string {
  if (!Number.isFinite(exalts)) return '0 ex';
  if (rate > 0 && Math.abs(exalts) >= rate) {
    return `${fmtNumber(exalts / rate, 2)} div`;
  }
  return `${fmtNumber(exalts, 1)} ex`;
}

export function fmtDivines(exalts: number, rate = divineRate): string {
  if (rate <= 0) return '0 div';
  return `${fmtNumber(exalts / rate, 2)} div`;
}
