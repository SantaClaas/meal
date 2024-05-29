import { clientOnly } from "@solidjs/start";
const ClientOnly = clientOnly(() => import("../components/test"));

export default function Home() {
  return (
    <>
      <h1>
        <ClientOnly />
      </h1>
    </>
  );
}
