{lib, ...}:
with lib;
with builtins;
with types; {
  # Example type
  email =
    addCheck str
    (str: match "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+.[a-zA-Z]{2,}" str != null);
}
