interface FeatureStatusProps {
  /** Whether the feature is on for the next or current session. */
  isOn: boolean;
  /** One sentence saying what is on or off, and what is sent where. */
  children: string;
}

/** One line telling the user whether a feature sends data to its provider. */
export function FeatureStatus({ isOn, children }: FeatureStatusProps) {
  return (
    <li className="feature" data-on={isOn}>
      <span className="feature-dot" aria-hidden="true" />
      {children}
    </li>
  );
}
