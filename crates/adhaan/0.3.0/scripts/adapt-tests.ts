const removeLeadingZero = (i: string): string => i.replace(/^0/, "");

function stringToTimeParam(s: string) {
  const [hm, ampm] = s.split(" ");
  const [h, m] = hm.split(":");
  const h24 =
    Number.parseInt(h) +
    (ampm === "PM" && h !== "12" ? 12 : ampm === "AM" && h === "12" ? -12 : 0);
  return `${h24}, ${removeLeadingZero(m)}`;
}

let moduleFileText = "";

const outputPath = "../src/vendored_tests";
await Deno.mkdir(outputPath, { recursive: true });

const testDataDir = "../testdata/Times";
for await (const testDataFile of Deno.readDir(testDataDir)) {
  const {
    params,
    times,
  }: {
    params: {
      latitude: number;
      longitude: number;
      timezone: string;
      method: string;
      madhab: "Shafi" | "Hanafi";
      highLatitudeRule: "MiddleOfTheNight";
    };
    times: {
      date: string;
      fajr: string;
      sunrise: string;
      dhuhr: string;
      asr: string;
      maghrib: string;
      isha: string;
      [prayer: string]: string;
    }[];
  } = JSON.parse(
    await Deno.readTextFile(testDataDir + "/" + testDataFile.name),
  );

  const testName = testDataFile.name
    .split(".")[0]
    .replaceAll(/[-\s]/g, "_")
    .toLowerCase();

  const testCasesByDate = times
    .map((testCase) => {
      const civilDate = testCase.date
        .split("-")
        .map(removeLeadingZero)
        .join(", ");

      const asserts = [
        ["Fajr", "fajr"],
        ["Sunrise", "sunrise"],
        ["Dhuhr", "dhuhr"],
        [params.madhab === "Shafi" ? "AsrAwwal" : "AsrThaani", "asr"],
        ["Maghrib", "maghrib"],
        ["Isha", "isha"],
      ]
        .map(
          ([prayerName, prayerTimeIndex]) => `
        let ${prayerTimeIndex} = date.at(${stringToTimeParam(testCase[prayerTimeIndex])}, 0, 0).in_tz("${params.timezone}").unwrap();
        let scheduled_${prayerTimeIndex} = schedule.time_of(Prayer::${prayerName});
        assert!(
            (&${prayerTimeIndex} - variance..=&${prayerTimeIndex} + variance).contains(&scheduled_${prayerTimeIndex}),
            "expected = {}\\nfound = {}",
            ${prayerTimeIndex},
            scheduled_${prayerTimeIndex},
        );`,
        )
        .join("\n");
      return `{
        let date = jiff::civil::date(${civilDate});
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();
${asserts}
    }`;
    })
    .join("\n    ");

  const method =
    params.method === "MoonsightingCommittee"
      ? "MOONSIGHTING_COMMITTEE"
      : params.method;

  const testOutputFileText = `use crate::*;

#[test]
fn for_${testName}() {
    let coordinates = Coordinates {
        latitude: ${params.latitude},
        longitude: ${params.longitude},
    };
    let params = Parameters::new(&prominent_methods::${method})
        .with_high_latitude_rule(HighLatitudeRule::${params.highLatitudeRule});
    let variance = jiff::SignedDuration::from_mins(2);

    ${testCasesByDate}
}
`;

  await Deno.writeTextFile(`${outputPath}/${testName}.rs`, testOutputFileText);

  moduleFileText += `mod ${testName};\n`;
}

await Deno.writeTextFile(`${outputPath}.rs`, moduleFileText);
