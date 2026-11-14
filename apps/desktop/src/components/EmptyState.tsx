// Honest placeholder: says what will appear here and in which stage,
// instead of showing invented numbers.
import { Icon, type IconName } from "./Icon";

type Props = { icon: IconName; title: string; text: string; stage: number };

export function EmptyState({ icon, title, text, stage }: Props) {
  return (
    <section className="empty-state">
      <div className="empty-icon">
        <Icon name={icon} size={28} />
      </div>
      <h2>{title}</h2>
      <p>{text}</p>
      <span className="stage-tag">Planned for stage {stage}</span>
    </section>
  );
}
