declare namespace Kali {
  function test(name: string, fn: () => void | Promise<void>): void;
}

declare module "@kali/test" {
  export function test(name: string, fn: () => void | Promise<void>): void;
}
