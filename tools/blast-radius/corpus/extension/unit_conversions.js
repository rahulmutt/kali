// Convert between SI-prefixed quantities and report a few derived physical
// values -- the sort of scratch calculation that gets committed as a script so
// nobody has to retype the constants.

const AVOGADRO = 6.02214076e23;
const PLANCK = 6.62607015e-34;
const ELECTRON_CHARGE = 1.602176634e-19;
const SPEED_OF_LIGHT = 299792458;
const BOLTZMANN = 1.380649e-23;

const PREFIXES = {
  nano: 1e-9,
  micro: 0.000001,
  milli: 0.001,
  unit: 1,
  kilo: 1000,
  mega: 1000000,
  giga: 1000000000,
};

function toBase(value, prefix) {
  const factor = PREFIXES[prefix];
  if (factor === undefined) {
    console.warn("unknown prefix: " + prefix);
    return value;
  }
  return value * factor;
}

function fromBase(value, prefix) {
  return value / PREFIXES[prefix];
}

function photonEnergy(wavelengthNanometres) {
  const metres = toBase(wavelengthNanometres, "nano");
  return (PLANCK * SPEED_OF_LIGHT) / metres;
}

function inElectronVolts(joules) {
  return joules / ELECTRON_CHARGE;
}

function thermalEnergy(kelvin) {
  return BOLTZMANN * kelvin;
}

function molesToParticles(moles) {
  return moles * AVOGADRO;
}

console.log("2.5 kilometres in metres:", toBase(2.5, "kilo"));
console.log("1500 metres in kilometres:", fromBase(1500, "kilo"));
console.log("particles in 0.25 mol:", molesToParticles(0.25).toExponential(4));

for (const wavelength of [405, 532, 650]) {
  const joules = photonEnergy(wavelength);
  console.log(
    wavelength + " nm -> " + joules.toExponential(3) + " J = " +
      inElectronVolts(joules).toFixed(3) + " eV",
  );
}

console.log("thermal energy at 300 K:", thermalEnergy(300).toExponential(3), "J");
console.log("thermal energy at 300 K:", inElectronVolts(thermalEnergy(300)).toFixed(5), "eV");
