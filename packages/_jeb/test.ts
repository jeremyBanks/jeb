// spellchecker: disable=all
import { assertEquals } from "jsr:@std/assert";
import { encodeJeb85 } from "jsr:@jeb/jeb";

function toUtf8(str: string): Uint8Array {
  return new TextEncoder().encode(str);
}

Deno.test({
  name: "encodeJeb85",
  fn() {
    const text = `\
But I must explain to you how all this mistaken idea of denouncing pleasure and
praising pain was born and I will give you a complete account of the system, and
expound the actual teachings of the great explorer of the truth, the
master-builder of human happiness. No one rejects, dislikes, or avoids pleasure
itself, because it is pleasure, but because those who do not know how to pursue
pleasure rationally encounter consequences that are extremely painful. Nor again
is there anyone who loves or pursues or desires to obtain pain of itself,
because it is pain, but because occasionally circumstances occur in which toil
and pain can procure him some great pleasure. To take a trivial example, which
of us ever undertakes laborious physical exercise, except to obtain some
advantage from it? But who has any right to find fault with a man who chooses to
 enjoy a pleasure that has no annoying consequences, or one who avoids a pain
 that produces no resultant pleasure?`;
    const expected = `\
i|But I must explain to you how all this mistaken idea of denouncing pleasure a\
nd|.............vqYP.j|praising pain was born and I will give you a complete ac\
count of the system, and|.................3t1E3f|ound the actual teachings of t\
he great explorer of the truth, the|............wEro.i|ster-builder of human ha\
ppiness. No one rejects, dislikes, or avoids pleasure|...............wErc[i|sel\
f, because it is pleasure, but because those who do not know how to pursue|....\
...........wErx)i|easure rationally encounter consequences that are extremely p\
ainful. Nor again|..............x(jn(h|s there anyone who loves or pursues or d\
esires to obtain pain of itself,|...............3sWNRh|ause it is pain, but bec\
ause occasionally circumstances occur in which toil|............z!T1{i|and pain\
 can procure him some great pleasure. To take a trivial example, which|........\
......v}Vi/g|f us ever undertakes laborious physical exercise, except to obtain\
 some|...........z/64)j|advantage from it? But who has any right to find fault \
with a man who chooses to|.................3lS[$h|joy a pleasure that has no an\
noying consequences, or one who avoids a pain|.............x(jn20|that produce\
s no resultant pleasure?`;

    const encoded = encodeJeb85(toUtf8(text));

    assertEquals(encoded, expected);
  },
});
