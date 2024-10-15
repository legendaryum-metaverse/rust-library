# `rust-library`

---

To generate the file

```bash
fd -e go . ./golang_library | xargs -I {} sh -c 'echo "// {}"; cat "{}"' | tee library.go
fd -e ts . ./packages/legend-transac/src/ | xargs -I {} sh -c 'echo "// {}"; cat "{}"' | tee library.ts
fd -e rs . ./src/ | xargs -I {} sh -c 'echo "// {}"; cat "{}"' | tee library.rs
```

---

## Finishing the app

Steps to publish the app

- cargo login ---> Se necesita una cuenta en crates.io, generar el token con permissos sufientes para publicar y actualizar versiones, luego verificar el email
- cargo package -p my-awesome-rabbitmq-lib
- cargo publish -p my-awesome-rabbitmq-lib

Pendientes:

- docs -> https://github.com/rust-lang/futures-rs/blob/7211cb7c5d8d859fa28ae55808c763a09d502827/.github/workflows/ci.yml#L306
- review Best Practices: https://www.cloudamqp.com/blog/part4-rabbitmq-13-common-errors.html
