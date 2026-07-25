import { useEffect, useState } from "react";

const eg_programs = import.meta.glob("./../../examples/*", {
    query: '?raw',
    import: 'default',
});

const default_program = "strings";

function ProgramDropdown({ setEditorValue }: { setEditorValue: (x: string) => void }) {
    const [egProgramsMap, setEgProgramsMap] = useState<Map<string, string>>(new Map());

    useEffect(() => {
        const loadPrograms = async () => {
            const eg_programs_map: Map<string, string> = new Map();
            for (const path in eg_programs) {
                const program = await eg_programs[path]();

                const eg_name = path.split('\\').pop()?.split('/').pop()?.replace(".sfl", "");
                if (eg_name == undefined) { continue; }

                // Skip programs that have the same value syntax as the local storage key
                if (eg_name.startsWith("__") && eg_name.endsWith("__")) { continue; }
                eg_programs_map.set(eg_name, program as string)
            }

            if (!eg_programs_map.has(default_program)) {
                console.error("Default program not found in eg_programs_map");
            }

            setEgProgramsMap(eg_programs_map);
            setEditorValue(eg_programs_map.get(default_program) || "");
        };
        loadPrograms()
    }, [setEditorValue]);

    const name_options: JSX.Element[] = [];
    egProgramsMap.forEach((_, name) => {
        if (name == default_program) {
            name_options.push(<option selected key={name} value={name}>{name}</option>);
        } else {
            name_options.push(<option key={name} value={name}>{name}</option>);
        }
    });
    const onChange = () => {
        const e = document.getElementById("program_dropdown") as HTMLSelectElement;
        const val = e?.value;
        if (val == "__local__") {
            setEditorValue(localStorage.getItem("program") || "");
        } else {
            setEditorValue(egProgramsMap.get(val) || "")
        }
    };

    return <select onChange={onChange} id="program_dropdown">
        <optgroup label="Inbuilt">{name_options}</optgroup>
        <option value="__local__">Local</option>
    </select>
}

interface ButtonsProps {
    handleRun: (programInput: string, multiple: boolean) => void;
    setEditorValue: (x: string) => void;
    editorValue: string;
    settingsIsVisible: boolean;
    setSettingsIsVisible: React.Dispatch<React.SetStateAction<boolean>>;
}

export default function Buttons({ handleRun, setEditorValue, editorValue, settingsIsVisible, setSettingsIsVisible }: ButtonsProps) {
    return <div id="Buttons">
        <ProgramDropdown setEditorValue={setEditorValue} />
        <button onClick={() => {setSettingsIsVisible(!settingsIsVisible); console.log("BLungus")}}>Settings</button>
        <button className="runbutton" id="RunButtonSingle" onClick={() => handleRun(editorValue, false)}>Lazy</button>
        <button className="runbutton" id="RunButtonMultiple" onClick={() => handleRun(editorValue, true)}>Free Choice</button>
    </div>
}