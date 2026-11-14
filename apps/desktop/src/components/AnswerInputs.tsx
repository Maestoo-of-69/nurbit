// Input controls for each answer kind. They only collect values; validation
// that matters happens again in Rust before anything is saved.
import type { AnswerValue, QuestionKind } from "../api";

const SCALE = Array.from({ length: 11 }, (_, i) => i);

type ScaleProps = {
  value: number | null;
  onChange: (value: number) => void;
  lowAnchor: string;
  highAnchor: string;
  label: string;
};

export function ScaleInput({ value, onChange, lowAnchor, highAnchor, label }: ScaleProps) {
  return (
    <div>
      <div className="scale" role="radiogroup" aria-label={label}>
        {SCALE.map((step) => (
          <button
            key={step}
            type="button"
            role="radio"
            aria-checked={value === step}
            className="scale-step"
            onClick={() => onChange(step)}
          >
            {step}
          </button>
        ))}
      </div>
      <div className="scale-anchors">
        <span>0 · {lowAnchor}</span>
        <span>10 · {highAnchor}</span>
      </div>
    </div>
  );
}

type OptionProps<T> = {
  options: { id: T; label: string }[];
  value: T | null;
  onChange: (value: T) => void;
  label: string;
};

function OptionButtons<T extends string | boolean>({ options, value, onChange, label }: OptionProps<T>) {
  return (
    <div className="options" role="radiogroup" aria-label={label}>
      {options.map((option) => (
        <button
          key={String(option.id)}
          type="button"
          role="radio"
          aria-checked={value === option.id}
          className="option"
          onClick={() => onChange(option.id)}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

type AnswerInputProps = {
  kind: QuestionKind;
  value: AnswerValue | null;
  onChange: (value: AnswerValue) => void;
  label: string;
};

/** Renders the right control for a follow-up question's kind. */
export function AnswerInput({ kind, value, onChange, label }: AnswerInputProps) {
  switch (kind.type) {
    case "scale":
      return (
        <ScaleInput
          label={label}
          value={value?.kind === "scale" ? value.value : null}
          onChange={(v) => onChange({ kind: "scale", value: v })}
          lowAnchor={kind.low_anchor}
          highAnchor={kind.high_anchor}
        />
      );
    case "yes_no":
      return (
        <OptionButtons
          label={label}
          options={[
            { id: true, label: "Yes" },
            { id: false, label: "No" },
          ]}
          value={value?.kind === "yes_no" ? value.value : null}
          onChange={(v) => onChange({ kind: "yes_no", value: v })}
        />
      );
    case "choice":
      return (
        <OptionButtons
          label={label}
          options={kind.options.map(([id, text]) => ({ id, label: text }))}
          value={value?.kind === "choice" ? value.value : null}
          onChange={(v) => onChange({ kind: "choice", value: v })}
        />
      );
    case "number":
      return (
        <label className="number-input">
          <input
            type="number"
            min={kind.min}
            max={kind.max}
            step={0.5}
            aria-label={label}
            value={value?.kind === "number" ? value.value : ""}
            onChange={(e) => {
              const n = e.target.valueAsNumber;
              if (Number.isFinite(n)) onChange({ kind: "number", value: n });
            }}
          />
          <span>{kind.unit}</span>
        </label>
      );
  }
}
