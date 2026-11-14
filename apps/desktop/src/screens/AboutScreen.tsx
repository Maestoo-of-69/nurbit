import type { AppInfo } from "../api";
import { ScreenHeader } from "../components/ScreenHeader";

type Props = { info: AppInfo | null };

export function AboutScreen({ info }: Props) {
  return (
    <>
      <ScreenHeader title="About" />
      <section className="panel panel-wide about">
        <h2>Nurbit {info ? info.version : ""}</h2>
        <p>
          Beta prototype of an autonomous astronaut health monitoring system, built for the NASA
          Space Apps Challenge.
        </p>
        <ul className="about-list">
          <li>Research and education prototype. Not a medical device.</li>
          <li>Not clinically validated and not certified for spaceflight or clinical use.</li>
          <li>All crew profiles and medical records are synthetic.</li>
          <li>The assistant and risk modules support medical specialists; they never replace them.</li>
        </ul>
      </section>
    </>
  );
}
