type ClassDictionary = Record<string, boolean | undefined | null>;
type ClassArray = ClassValue[];
type ClassValue = string | null | undefined | false | ClassDictionary | ClassArray;

const isClassDictionary = (value: ClassValue): value is ClassDictionary =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const isClassArray = (value: ClassValue): value is ClassArray => Array.isArray(value);

const collectClasses = (value: ClassValue, acc: string[]): void => {
  if (!value) return;

  if (typeof value === "string") {
    acc.push(value);
    return;
  }

  if (isClassArray(value)) {
    value.forEach((item) => collectClasses(item, acc));
    return;
  }

  if (isClassDictionary(value)) {
    Object.entries(value).forEach(([key, enabled]) => {
      if (enabled) acc.push(key);
    });
  }
};

export const cn = (...values: ClassValue[]): string => {
  const classes: string[] = [];
  values.forEach((value) => collectClasses(value, classes));
  return classes.join(" ").trim();
};
