// Duty-restriction recommendations waiting for a SECOND specialist.
// The author sees their own items but cannot confirm them (checked in Rust too).
import { useState } from "react";
import { pendingReviews, reviewDecision, type CrewMember, type PendingReview, type Verdict } from "../../api";
import { EmptyState } from "../../components/EmptyState";
import { ScreenHeader } from "../../components/ScreenHeader";
import { useCommand } from "../../useCommand";

type Props = { actor: CrewMember; refreshKey: number; onChanged: () => void };

export function PendingReviewsScreen({ actor, refreshKey, onChanged }: Props) {
  const [version, setVersion] = useState(0);
  const items = useCommand(() => pendingReviews(actor.id), [actor.id, refreshKey, version]);
  const done = () => {
    setVersion((v) => v + 1);
    onChanged();
  };

  return (
    <>
      <ScreenHeader title="Pending reviews" subtitle="Recommendations that need a second specialist before they take effect." />
      {items.error && <p className="error">{items.error}</p>}
      {items.data?.length === 0 && (
        <section className="panel panel-wide">
          <p className="panel-text">Nothing waiting for a second review.</p>
        </section>
      )}
      {items.data === null && !items.error && <EmptyState icon="inbox" title="Loading" text="" stage={9} />}
      {items.data?.map((item) => (
        <ReviewCard key={item.decision.id} actor={actor} item={item} onDone={done} />
      ))}
    </>
  );
}

function ReviewCard({ actor, item, onDone }: { actor: CrewMember; item: PendingReview; onDone: () => void }) {
  const [comment, setComment] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  async function decide(verdict: Verdict) {
    setIsBusy(true);
    setError(null);
    try {
      await reviewDecision(actor.id, item.decision.id, verdict, comment);
      onDone();
    } catch (e: unknown) {
      setError(String(e));
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <section className="panel panel-wide human-panel">
      <div className="panel-heading">
        <div className="panel-label">
          {item.member_name} · recommended by {item.author_name}
        </div>
        <span className="cell-note">{new Date(item.decision.created_at).toLocaleString("en-GB")}</span>
      </div>
      <p>
        <strong>Recommend removal from operational duty.</strong> {item.decision.text}
      </p>
      {item.can_review ? (
        <>
          <textarea className="text-input" placeholder="Comment (optional)" value={comment} onChange={(e) => setComment(e.target.value)} />
          <div className="button-row">
            <button type="button" className="button-primary" disabled={isBusy} onClick={() => decide("confirm")}>
              Confirm
            </button>
            <button type="button" className="button-secondary" disabled={isBusy} onClick={() => decide("reject")}>
              Reject
            </button>
          </div>
        </>
      ) : (
        <p className="cell-note">You wrote this recommendation. Another specialist must review it.</p>
      )}
      {error && <p className="error">{error}</p>}
    </section>
  );
}
