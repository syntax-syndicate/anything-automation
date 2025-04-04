import { createHeadlessForm } from "@remoteoss/json-schema-form";
import {
  CREATE_VARIABLE_SCHEMA,
  EDIT_VARIABLES_VARIABLES,
} from "./create-variable-schema";
import { JsonSchemaForm } from "../json-schema-form";
import { useAnything } from "@/context/AnythingContext";
import { EditVariableFormMode } from "@/context/VariablesContext";
import { useMemo } from "react";

function extractObjectValues(
  obj: Record<string, any> | null,
  keys: string[],
): Record<string, any> {
  if (obj === null) {
    return {};
  }

  const extractedObject: Record<string, any> = {};
  keys.forEach((key) => {
    if (obj.hasOwnProperty(key)) {
      extractedObject[key] = obj[key];
    }
  });
  return extractedObject;
}

// Edit a single variable
export default function CreateVariableForm(): JSX.Element {
  const { workflow, variables } = useAnything();

  let the_variable = {
    ...EDIT_VARIABLES_VARIABLES,
    ...extractObjectValues(
      variables.selectedProperty,
      Object.keys(EDIT_VARIABLES_VARIABLES),
    ),
  };

  console.log("The Variable", the_variable);

  if (
    variables.selectedProperty !== null &&
    variables.selectedProperty !== undefined
  ) {
    //Trick from into thining we cannot edit Title.
    //We don't really want users changing it. Seems it might make things more brittle
    CREATE_VARIABLE_SCHEMA.properties.title.default =
      variables.selectedProperty.title;
    CREATE_VARIABLE_SCHEMA.properties.title.const =
      variables.selectedProperty.title;
    //https://json-schema-form.vercel.app/?path=/docs/guides-concepts-forced-values--docs
  } else {
    delete CREATE_VARIABLE_SCHEMA.properties.title.default;
    delete CREATE_VARIABLE_SCHEMA.properties.title.const;
  }

  const { fields, handleValidation } = useMemo(() => {
    let results = createHeadlessForm(CREATE_VARIABLE_SCHEMA, {
      initialValues: the_variable,
    });
    //Used to add "strict" feature kinda sideloading jsonschema form or our validation needs
    results.fields = results.fields.map((field: any) => ({
      ...field,
      strict:
        CREATE_VARIABLE_SCHEMA.properties[field.name]?.["x-any-validation"]
          ?.strict || true,
    }));

    return results;
  }, [the_variable]);

  async function handleOnSubmit(formValues: any) {
    console.log(
      "[VARIABLES CONTEXT FORM]Submitting newly created variables",
      formValues,
    );
    await variables.updateVariablesProperty(formValues);
    console.log("Submitted!", formValues);
    variables.setEditingMode(EditVariableFormMode.INPUT);
  }

  return (
    <JsonSchemaForm
      name="edit-single-variable-form"
      onSubmit={handleOnSubmit}
      fields={fields}
      initialValues={the_variable}
      handleValidation={handleValidation}
    />
  );
}
