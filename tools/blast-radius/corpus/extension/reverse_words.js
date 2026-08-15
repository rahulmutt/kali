// Reverse the word order of a sentence in place over a character array, and
// reverse each word back -- the classic two-pointer string exercise, written
// the way it is written when the buffer is the thing you are given.

function reverseRange(chars, from, to) {
  for (let i = from, j = to; i < j; i++, j--) {
    const swap = chars[i];
    chars[i] = chars[j];
    chars[j] = swap;
  }
}

function reverseWordOrder(sentence) {
  const chars = sentence.split("");
  reverseRange(chars, 0, chars.length - 1);

  let start = 0;
  for (let i = 0; i <= chars.length; i++) {
    if (i === chars.length || chars[i] === " ") {
      reverseRange(chars, start, i - 1);
      start = i + 1;
    }
  }
  return chars.join("");
}

function isPalindrome(text) {
  const cleaned = text.toLowerCase().split("").filter((ch) => ch >= "a" && ch <= "z");
  let i = 0;
  let j = cleaned.length - 1;
  for (; i < j; i++, j--) {
    if (cleaned[i] !== cleaned[j]) return false;
  }
  return true;
}

const SENTENCE = "the tree becomes code before the code becomes bytes";
console.log(reverseWordOrder(SENTENCE));
console.log("round trip:", reverseWordOrder(reverseWordOrder(SENTENCE)) === SENTENCE);

for (const candidate of ["A man, a plan, a canal: Panama", "not a palindrome"]) {
  console.log(candidate + " -> " + String(isPalindrome(candidate)));
}
