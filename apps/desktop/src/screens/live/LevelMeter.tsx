import { Icon, type IconName } from "../../components/Icon";
import type { Level } from "../../capture/captureClient";

interface LevelMeterProps {
  label: string;
  icon: IconName;
  level: Level;
}

/** A bar showing how loud one audio source currently is. */
export function LevelMeter({ label, icon, level }: LevelMeterProps) {
  return (
    <label className="level">
      <Icon name={icon} />
      <span className="visually-hidden">{label}</span>
      <meter min={0} max={1} value={level.peak} />
    </label>
  );
}
