// One 0-10 rating over time, with the person's own baseline band.
// Single series (title names it, no legend); 2px line, 8px markers;
// filled markers = self-report, hollow = synthetic history (shape, not colour);
// hover tooltip per point. Plain SVG, no chart library.
import { useState } from "react";
import type { Baseline, TrendPoint } from "../api";

const W = 640;
const H = 170;
const PAD = { left: 28, right: 12, top: 10, bottom: 22 };
const MAX = 10;

type Props = { label: string; points: TrendPoint[]; baseline: Baseline | null };

const fmtDate = (ms: number) => new Date(ms).toLocaleDateString("en-GB", { day: "numeric", month: "short" });

export function TrendChart({ label, points, baseline }: Props) {
  const [hover, setHover] = useState<number | null>(null);
  if (points.length === 0) return <p className="cell-note">No values yet.</p>;

  const innerW = W - PAD.left - PAD.right;
  const innerH = H - PAD.top - PAD.bottom;
  const x = (i: number) => PAD.left + (points.length === 1 ? innerW / 2 : (i / (points.length - 1)) * innerW);
  const y = (v: number) => PAD.top + innerH - (Math.max(0, Math.min(MAX, v)) / MAX) * innerH;
  const path = points.map((p, i) => `${i === 0 ? "M" : "L"}${x(i)},${y(p.value)}`).join(" ");
  const step = points.length > 1 ? innerW / (points.length - 1) : innerW;
  const hovered = hover === null ? null : points[hover];

  return (
    <div className="trend-chart">
      <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label={`${label} over time, 0 to 10`}>
        {[0, 5, 10].map((v) => (
          <g key={v}>
            <line className="chart-grid" x1={PAD.left} x2={W - PAD.right} y1={y(v)} y2={y(v)} />
            <text className="chart-axis" x={PAD.left - 8} y={y(v) + 4} textAnchor="end">
              {v}
            </text>
          </g>
        ))}
        {baseline && (
          <>
            <rect
              className="chart-band"
              x={PAD.left}
              width={innerW}
              y={y(Math.min(MAX, baseline.high))}
              height={y(Math.max(0, baseline.low)) - y(Math.min(MAX, baseline.high))}
            />
            <line className="chart-mean" x1={PAD.left} x2={W - PAD.right} y1={y(baseline.mean)} y2={y(baseline.mean)} />
          </>
        )}
        <path className="chart-line" d={path} />
        {hover !== null && (
          <line className="chart-crosshair" x1={x(hover)} x2={x(hover)} y1={PAD.top} y2={H - PAD.bottom} />
        )}
        {points.map((p, i) => (
          <circle
            key={p.at}
            className={p.simulated ? "chart-dot chart-dot-hollow" : "chart-dot"}
            cx={x(i)}
            cy={y(p.value)}
            r={4}
          />
        ))}
        {points.map((p, i) => (
          <rect
            key={`hit-${p.at}`}
            className="chart-hit"
            x={x(i) - step / 2}
            width={step}
            y={PAD.top}
            height={innerH}
            onMouseEnter={() => setHover(i)}
            onMouseLeave={() => setHover(null)}
          />
        ))}
        <text className="chart-axis" x={PAD.left} y={H - 4}>
          {fmtDate(points[0]!.at)}
        </text>
        <text className="chart-axis" x={W - PAD.right} y={H - 4} textAnchor="end">
          {fmtDate(points[points.length - 1]!.at)}
        </text>
      </svg>
      {hovered && hover !== null && (
        <div className="chart-tooltip" style={{ left: `${(x(hover) / W) * 100}%` }}>
          <strong>{hovered.value}</strong> / 10 · {fmtDate(hovered.at)}
          <span>{hovered.simulated ? "Synthetic history" : "Self-report"}</span>
        </div>
      )}
    </div>
  );
}
