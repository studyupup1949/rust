# adejava
AdeJava es un proyecto para Java EE 7

La tecnología de servlet en Java EE 7 se ha actualizado para incluir características que ayudan a incorporar la tecnología para su uso con tecnologías más nuevas, como HTML 5. La versión anterior, Servlet 3.0, se centró en hacer que los servlets sean más fáciles de escribir y administrar. Tales actualizaciones
como configuración web.xml opcional, el procesamiento de solicitudes asíncronas y la incorporación de la tecnología FileUpload hacen que la escritura de servlets sea mucho más fácil que antes. 

La versión de Servlet 3.1 trae más mejoras para ayudar a poner la tecnología en línea con las tecnologías actuales, incluidas las siguientes:

- Se basa en las nuevas características de Servlet 3.0, presentando la API de E / S sin bloqueo
- La capacidad de trabajar con el protocolo WebSockets.
- Incluye mejoras de seguridad
- Ofrece otras actualizaciones diversas

## E / S sin bloqueo

La tecnología de servlet ha permitido solo la entrada / salida tradicional (bloqueo) durante el procesamiento de solicitudes desde su inicio. En la versión Servlet 3.1, la nueva API de E / S sin bloqueo hace posible que los servlets lean o escriban sin ningún bloqueo. Esto significa que se pueden realizar otras tareas al mismo tiempo que se realiza una lectura o escritura, sin esperar. Esto, a su vez, significa que ahora puede realizar Ajax y actualizaciones de página parciales más fácilmente sin realizar llamadas separadas al servlet para cada actualización. Esta solución abre un nuevo campo de posibilidades para los servlets, haciéndolos mucho más flexibles para su uso junto con tecnologías modernas como el [protocolo WebSockets](https://bitcu.co/192-168-8-1/).

Para implementar una solución de E / S sin bloqueo, se han agregado nuevas interfaces de programación a ServletInputStream y ServletOutputStream, así como dos detectores de eventos: ReadListener y WriteListener. Las interfaces ReadListener y WriteListener hacen que el procesamiento de E / S del servlet se produzca sin bloqueo a través de métodos de devolución de llamada que se invocan cuando el contenedor puede hacerlo sin bloquear otros procesos. Use el método ServletInputStream.setReadListener (ServletInputStream, AsyncContext) para registrar un ReadListener con un ServletInputStream, y use el método E / S de lectura ServletInputStream.setWriteListener (ServletOutputStream, AsyncContext) para registrar un WriteListener. 

Las siguientes líneas de código demuestran cómo registrar una implementación de ReadListener con un ServletInputStream:

AsyncContext context = request.startAsync(); 
ServletInputStream input = request.getInputStream(); 
input.setReadListener (new ReadListenerImpl(entrada, contexto));

En Servlet 3.0, AsyncContext se introdujo para representar un contexto de ejecución para una operación asincrónica que se inicia en una solicitud de servlet. para usar el contexto asincrónico, un servlet debe anotarse como @WebServlet y el atributo asyncSupported de la anotación debe establecerse en verdadero. la anotación @WebFilter también contiene el atributo asyncSupported.

Veamos cómo realizar una lectura sin bloqueo. Después de que se haya registrado un oyente
con un ServletInputStream, el estado de una lectura sin bloqueo se puede verificar llamando a los métodos ServletInputStream.isReady y ServletInputStream.isFinished. 
Por ejemplo, una lectura puede comenzar una vez que el método ServletInputStream.isReady devuelve un verdadero, como se muestra aquí:

while (is.isReady () && (b = input.read ())! = -1)) 
{
  len = is.read (b);
  String data = new String (b, 0, len);
}

Para crear un ReadListener o WriteListener, se deben anular tres métodos: onDataAvailable, onAllDataRead y onError. El método onDataAvailable se invoca cuando los datos están disponibles para ser leídos o
escrito, onAllDataRead se invoca una vez que se han leído o escrito todos los datos, y se invoca onError si se encuentra un error. El código demuestra una forma de implementar la interfaz ReadListener. 
