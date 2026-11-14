// Daily check: 3 mandatory questions -> follow-ups chosen by Rust rules
// (with the reason shown) -> save -> summary. Questions, rules, validation
// and storage all live in Rust; this screen only collects answers.
import { useRef, useState } from "react";
import {
  getDailyQuestions,
  planFollowUps,
  saveCheckin,
  type AnswerValue,
  type CrewMember,
  type DailyScores,
  type FollowUpPlan,
  type SaveResult,
} from "../api";
import { AnswerInput, ScaleInput } from "../components/AnswerInputs";
import { ScreenHeader } from "../components/ScreenHeader";
import { useCommand } from "../useCommand";

type Step = "daily" | "follow-ups" | "done";
const DAILY_KEYS: (keyof DailyScores)[] = ["rested", "physical", "mood_energy"];

type Props = { member: CrewMember; onSaved: () => void };

export function DailyCheckScreen({ member, onSaved }: Props) {
  const questions = useCommand(getDailyQuestions);
  const startedAt = useRef(Date.now());
  const [step, setStep] = useState<Step>("daily");
  const [scores, setScores] = useState<Partial<DailyScores>>({});
  const [plan, setPlan] = useState<FollowUpPlan | null>(null);
  const [followUps, setFollowUps] = useState<Record<string, AnswerValue>>({});
  const [result, setResult] = useState<SaveResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  if (member.role !== "astronaut") {
    return (
      <>
        <ScreenHeader title="Daily check" />
        <div className="callout">Daily checks are completed by astronauts. The specialist view arrives in stage 9.</div>
      </>
    );
  }

  const complete = DAILY_KEYS.every((k) => scores[k] !== undefined);
  const allFollowUpsAnswered = plan?.questions.every((q) => followUps[q.question.id]) ?? true;

  async function run(action: () => Promise<void>) {
    setIsBusy(true);
    setError(null);
    try {
      await action();
    } catch (e: unknown) {
      setError(String(e));
    } finally {
      setIsBusy(false);
    }
  }

  async function save(daily: DailyScores, answers: Record<string, AnswerValue>) {
    const submitted = Object.entries(answers).map(([question_id, value]) => ({ question_id, value }));
    setResult(await saveCheckin(member.id, startedAt.current, daily, submitted));
    setStep("done");
    onSaved();
  }

  const handleContinue = () =>
    run(async () => {
      const daily = scores as DailyScores;
      const planned = await planFollowUps(member.id, daily);
      if (planned.questions.length === 0) {
        await save(daily, {});
      } else {
        setPlan(planned);
        setStep("follow-ups");
      }
    });

  if (step === "done" && result) {
    return <CheckSummary result={result} />;
  }

  return (
    <>
      <ScreenHeader
        title="Daily check"
        subtitle={
          step === "daily"
            ? "Three mandatory questions. 0 = worst, 10 = best on every scale."
            : "A few more questions, because of today's answers."
        }
      />
      {error && <p className="error">{error}</p>}

      {step === "daily" && (
        <ol className="question-list">
          {questions.data?.map((q, index) => {
            const key = DAILY_KEYS[index];
            if (!key) return null;
            return (
              <li key={q.id} className="question-card">
                <div className="question-number">{index + 1}</div>
                <div className="question-body">
                  <div className="question-text">{q.text}</div>
                  <ScaleInput
                    label={q.text}
                    value={scores[key] ?? null}
                    onChange={(v) => setScores((s) => ({ ...s, [key]: v }))}
                    lowAnchor={q.low_anchor}
                    highAnchor={q.high_anchor}
                  />
                </div>
              </li>
            );
          })}
        </ol>
      )}

      {step === "follow-ups" && plan && (
        <>
          <div className="reasons">
            {plan.triggered.map((rule) => (
              <div key={rule.rule_id} className="reason">
                <strong>Why:</strong> {rule.reason}{" "}
                <span className="basis">Rule threshold: prototype assumption, needs specialist validation.</span>
              </div>
            ))}
          </div>
          <ol className="question-list">
            {plan.questions.map(({ question }, index) => (
              <li key={question.id} className={`question-card ${question.is_safety ? "question-safety" : ""}`}>
                <div className="question-number">{index + 1}</div>
                <div className="question-body">
                  {question.is_safety && <div className="safety-tag">Safety question</div>}
                  <div className="question-text">{question.text}</div>
                  <AnswerInput
                    label={question.text}
                    kind={question.kind}
                    value={followUps[question.id] ?? null}
                    onChange={(v) => setFollowUps((f) => ({ ...f, [question.id]: v }))}
                  />
                </div>
              </li>
            ))}
          </ol>
        </>
      )}

      <div className="form-actions">
        {step === "daily" && (
          <button type="button" className="button-primary" disabled={!complete || isBusy} onClick={handleContinue}>
            Continue
          </button>
        )}
        {step === "follow-ups" && (
          <button
            type="button"
            className="button-primary"
            disabled={!allFollowUpsAnswered || isBusy}
            onClick={() => run(() => save(scores as DailyScores, followUps))}
          >
            Save check-in
          </button>
        )}
      </div>
    </>
  );
}

function CheckSummary({ result }: { result: SaveResult }) {
  const { outcome } = result;
  const seconds = Math.round(result.duration_ms / 1000);
  const urgent = outcome.flags.filter((f) => f.flag === "urgent_symptoms");
  const other = outcome.flags.filter((f) => f.flag !== "urgent_symptoms");

  return (
    <>
      <ScreenHeader title="Check-in saved" subtitle="Stored on this computer and queued for the medical team." />
      {urgent.map((f) => (
        <div key={f.flag} className="alert-banner" role="alert">
          {f.message}
        </div>
      ))}
      <div className="summary-grid">
        <Stat label="Time taken" value={`${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`} />
        <Stat label="Questions asked" value={String(outcome.questions_asked)} />
        <Stat label="Follow-ups triggered" value={String(outcome.follow_ups_asked)} />
      </div>
      {other.length > 0 && (
        <section className="panel panel-wide">
          <div className="panel-label">Notes for the medical team</div>
          <ul className="plain-list">
            {other.map((f) => (
              <li key={f.flag}>{f.message}</li>
            ))}
          </ul>
        </section>
      )}
      {outcome.flags.length === 0 && (
        <p className="muted-text">No notifications. Normal results never create alerts.</p>
      )}
    </>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <section className="panel">
      <div className="panel-label">{label}</div>
      <div className="panel-value">{value}</div>
    </section>
  );
}
