func (c Controller) Write(w http.ResponseWriter,r *http.Request) {
 var body struct{Message *string `json:"message"`}
 decoder:=json.NewDecoder(http.MaxBytesReader(w,r.Body,8192))
 if err:=decoder.Decode(&body);err!=nil || body.Message==nil || len([]rune(*body.Message))<1 || len([]rune(*body.Message))>200 {http.Error(w,"message precisa conter 1 a 200 caracteres",http.StatusBadRequest);return}
 var trailing any
 if err:=decoder.Decode(&trailing);err!=io.EOF {http.Error(w,"JSON inválido",http.StatusBadRequest);return}
 writer,ok:=c.Service.(interface{Write(context.Context,string)(Example,error)})
 if !ok {http.Error(w,"Escrita indisponível",http.StatusInternalServerError);return}
 result,err:=writer.Write(r.Context(),*body.Message)
 if err!=nil {http.Error(w,"Falha ao persistir exemplo",http.StatusInternalServerError);return}
 w.Header().Set("Content-Type","application/json");_=json.NewEncoder(w).Encode(result)
}
