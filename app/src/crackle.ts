/**
 * An ergonomic way to messsage the service worker. This has edge cases so it is not ready for a library but good enough for now.
 * Inspired by comlink.
 */

// This type is written using GPT-4.1
type Promisify<T> = {
  [K in keyof T]: T[K] extends (...args: any[]) => infer R
    ? R extends Promise<any>
      ? // Function already returns Promise, leave as-is
        T[K]
      : // Wrap function return type
        (...args: Parameters<T[K]>) => Promise<R>
    : T[K] extends Promise<any>
    ? // Property already is Promise, leave as-is
      T[K]
    : // Wrap non-function, non-Promise property
      Promise<T[K]>;
};

type CrackleMessage = {
  property: string | symbol;
  parameters: unknown[];
};

/**
 * Proxy requests to the provided target object to the service worker
 */
export function expose<T extends object>(target: T, port: MessagePort) {
  port.addEventListener(
    "message",
    async (event: MessageEvent<CrackleMessage>) => {
      const { property, parameters } = event.data;
      const responsePort = event.ports[0];
      const targetProperty = Reflect.get(target, property, target);
      if (typeof targetProperty !== "function") {
        responsePort.postMessage(targetProperty);
        return;
      }

      const result = Reflect.apply(targetProperty, target, parameters);
      if (!(result instanceof Promise)) {
        responsePort.postMessage(result);
        return;
      }

      const response = await result;
      responsePort.postMessage(response);
    }
  );
}

export function wrap<T extends object>(port: MessagePort): Promisify<T> {
  const handler: ProxyHandler<T> = {
    get(target, property, receiver) {
      // When the proxy gets returned in a promise, that promise checks for the existence of the "then" property
      // If it exists it will treat this proxy as promise but we don't want that.
      // This enables JS async functions to return a promise without that promise gettting wrapped in a promise itself.
      // But the logic for that sadly checks for the existence of the "then" property on the proxy
      if (property === "then") return undefined;

      return async (...parameters: unknown[]) => {
        const { port1, port2 } = new MessageChannel();
        const response = new Promise<unknown>((resolve) => {
          port1.addEventListener("message", (event) => resolve(event.data), {
            once: true,
          });
        });

        port.postMessage({ property, parameters } satisfies CrackleMessage, [
          port2,
        ]);
        port1.start();
        return await response;
      };
    },
  };

  // We have to lie to TypeScript here that this will work
  return new Proxy({}, handler) as Promisify<T>;
}
