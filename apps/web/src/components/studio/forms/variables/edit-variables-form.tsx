import { Button } from "@/components/ui/button";
import { Edit2 } from "lucide-react";
import { useAnything } from "@/context/AnythingContext";
import { EditVariableFormMode } from "@/context/VariablesContext";
import DeleteVariableDialog from "./delete-variable-dialog";


export default function EditVariablesForm() {
    const { variables, workflow: { selected_node_variables_schema } } = useAnything();

    const handleEdit = (property: any | undefined) => {
        console.log("Create Variable");
        variables.setSelectedProperty(property);
        variables.setEditingMode(EditVariableFormMode.EDIT);
    }

    return (
        <div className="space-y-2 mt-4">
            <Button variant="default" className="w-full" onClick={() => handleEdit(null)}>Add Variable</Button>
            {selected_node_variables_schema && selected_node_variables_schema.properties && Object.keys(selected_node_variables_schema.properties).map((key: string) => (
                <div key={key} className="rounded-lg border p-1 flex flex-row align-center ">
                    <h2 className="flex items-center text-xl text-left w-full ">{selected_node_variables_schema.properties[key].title}</h2>
                    <div className="flex-1" />
                    <Button variant="outline" size="sm" className="ml-2" onClick={() => handleEdit({ ...selected_node_variables_schema.properties[key], key })}>
                        <Edit2 className="size-5" />
                    </Button>
                    <DeleteVariableDialog key={key} variable={{ ...selected_node_variables_schema.properties[key], key }} />
                </div>
            ))}
        </div>
    );
}
