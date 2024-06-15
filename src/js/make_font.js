import { createWriteStream, readFileSync } from 'fs';
import PNGReader from 'png.js';

process.env.DEBUG = true;

String.prototype.chunk = function (n) {
  return this.match(new RegExp('.{1,' + n + '}', 'g'));
}

const noop = () => { }, debug = process.env.DEBUG ? console.log : noop;

////////////////////////////////////////////////////////////////

const char_width = 7;
const char_height = 12;

const chars = [];
for (let i = 33; i < 127; i++) chars.push(String.fromCharCode(i));
for (let i = 161; i < 232; i++) chars.push(String.fromCharCode(i));

console.log(`loading image...`);

const img = await new Promise(resolve => {
  const reader = new PNGReader(readFileSync('src/font.png'));
  reader.parse((err, png) => {
    if (err) throw err;
    resolve(png);
  });
});

const bytes = Uint32Array.from({ length: 16 * 1323 });
for (let y = 0; y < 16; y++) {
  let line = '';
  for (let x = 0; x < 1323; x++) {
    const [r, g, b, a] = img.getPixel(x, y);
    console.log(`[${x},${y}]: ${r},${g},${b},${a}`);
    bytes[y + 1323 + x] = a * 16777216 + r * 65536 + g * 256 + b;
  }
}

// console.log(`processing image...`, bytes.join(','));




/* 

const map = new Array(256).fill(0);

for (let i = 0; i < chars.length; i++) {
  const char = chars[i];
  const ox = i * char_width;
  let byte = '';
  for (let y = 0; y < char_height; y++) {
    for (let x = 0; x < char_width; x++) {
      const [r, g, b] = ctx.getImageData(ox + x, y, 1, 1).data;
      const pixel_is_black = r + g + b != 765;
      byte += pixel_is_black ? '1' : '0';
    }
  }
  // debug(`[${i}]: \n${byte.toString(2).padStart(8, '0').chunk(char_width).join('\n').replace(/0/g, ' ').replace(/1/g, '█')}`);
  const number = BigInt('0b' + byte);
  const ord = char.charCodeAt(0);
  debug(`[${i}:${char}]: ${byte} => ${number} = ${ord}`);
  map[ord] = number;
}

const out = createWriteStream('test.png')
const stream = canvas.createPNGStream()
stream.pipe(out)
out.on('finish', () => console.log('The PNG file was created.'))

const test_text = 'Hello, World!';

console.log(`
ivec2 font_size = ivec2(${char_width}, ${char_height});
int font[256] = int[256](${map.join(', ')});
uint test_text[${test_text.length}] = uint8_t[${test_text.length}](${test_text.split('').map(c => c.charCodeAt(0)).join(', ')});
`);

 */