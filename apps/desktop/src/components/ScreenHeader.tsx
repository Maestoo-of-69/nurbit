import type { ReactNode } from "react";

type Props = { title: string; subtitle?: string; actions?: ReactNode };

export function ScreenHeader({ title, subtitle, actions }: Props) {
  return (
    <header className="screen-header">
      <div>
        <h1>{title}</h1>
        {subtitle && <p className="screen-subtitle">{subtitle}</p>}
      </div>
      {actions && <div className="screen-actions">{actions}</div>}
    </header>
  );
}
