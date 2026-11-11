import { useId } from "react";

interface RadioGroupProps<T extends string> {
  legend: string;
  options: readonly T[];
  value: T;
  onChange: (value: T) => void;
  optionLabel: (option: T) => string;
  /** Options the user cannot choose right now. */
  disabledOptions?: readonly T[];
}

/** A labelled group of radio buttons for choosing one of a few options. */
export function RadioGroup<T extends string>({
  legend,
  options,
  value,
  onChange,
  optionLabel,
  disabledOptions = [],
}: RadioGroupProps<T>) {
  const name = useId();
  return (
    <fieldset className="field radio-group">
      <legend>{legend}</legend>
      {options.map((option) => (
        <label key={option} className="choice">
          <input
            type="radio"
            name={name}
            checked={option === value}
            disabled={disabledOptions.includes(option)}
            onChange={() => {
              onChange(option);
            }}
          />
          {optionLabel(option)}
        </label>
      ))}
    </fieldset>
  );
}
