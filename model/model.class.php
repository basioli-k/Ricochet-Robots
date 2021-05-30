<?php

require_once __SITE_PATH . "/model/player.class.php";

abstract class Model {

    protected static $table = null;

    protected $columns = [];

    protected static $attributes = [];

    public function __get( $col )
    {
        if( isset( $this->columns[ $col ] ) )
            return $this->columns[ $col ];

        return null;
    }

    public function __set( $col, $value )
    {
        $this->columns[$col] = $value;

        return $this;
    }

    static function generateAttributeList($put_id = true){
        $first = true;
        $query_part = "";
        foreach(static::$attributes as $attr => $type){
            if($first){
                $first = false;
                $query_part .= $attr;
            }   
            else
                $query_part .= ", " . $attr;
        }

        return $query_part;
    }

    public static function all()
    {
        $db = DB::getConnection();

        $query = "SELECT " . generateAttributeList() . " FROM " . static::$table . ";";

        $st = $db->prepare($query);

        try{
            $st->execute();
        }
        catch( PDOException $e ) { exit( "PDO error [select all from " . static::$table . "]: " . $e->getMessage() ); }

        $class_name = get_called_class();
        $result = array();

        foreach($st->fetchAll() as $row)
            array_push($result, new $class_name($row) );
        
        return $result;

    }

    public static function find( $id )
    {
        $db = DB::getConnection();

        $st = $db->prepare("SELECT * FROM " . static::$table . " WHERE id=" . ":id;");

        try{
            $st->execute(array("id" => $id));
        }
        catch( PDOException $e ) { exit( "PDO error [select from " . static::$table . "]: " . $e->getMessage() ); }

        //always exactly one
        return $st->fetch();
    }

    public static function where( $query_array )
    {
        $db = DB::getConnection();

        $query = "SELECT " . static::generateAttributeList() . " FROM " . static::$table . " WHERE ";

        $first = true;

        foreach($query_array as $column => $value){
            if($first){
                $query = $query . $column . "=:" .$column;
                $first = false;
            }
            else
                $query = $query . " AND " . $column . "=:" . $column;
                
        }
        
        $query = $query . ";";

        $st = $db->prepare($query);
        try{
            $st->execute($query_array);
        }
        catch( PDOException $e ) { exit( "PDO error [select from " . static::$table . "]: " . $e->getMessage() ); }

        $class_name = get_called_class();
        $result = array();

        foreach($st->fetchAll() as $row)
            array_push($result, new $class_name($row) );
        
        return $result;

    }

    public static function insert( $row )
    {
        $query = "INSERT INTO " . static::$table . "(";  
        
        $first = true;
        foreach($row as $key => $value){
            if($first){
                $query .= $key;
                $first = false;
            }
            else   
                $query .= ", " . $key;   
        }
        
        $query .= ")";
        
        $query .= " VALUES";
        $first = true;

        foreach($row as $key => $value){
            if($first){
                $query .= "(:" . $key;
                $first = false;
            }
            else   
                $query .= ", :" . $key;   
        }

        $query .= ");";
        
        $db = DB::getConnection();
        
        $st = $db->prepare( $query );
        try{
            $st->execute( $row );
        }
        catch( PDOException $e ) { exit( "PDO error [insert into " . static::$table . "]: " . $e->getMessage() ); }

        return true;
    }

    public function update()
    {
        $query = "UPDATE " . static::$table . " SET ";
        
        $first = true;
        
        $row = array();

        foreach(static::$attributes as $attr => $value){
            if($first){
                $query .= $attr . "=:" . $attr;
                $first = false;
            }
            else
                $query .= ", " . $attr . "=:" . $attr;
            
            $row[$attr] = $this->$attr;
        }

        $query = $query . " WHERE id=:id;";

        $db = DB::getConnection();

        $st = $db->prepare( $query );
        
        try{
            $st->execute( $row );
        }
        catch( PDOException $e ) { exit( "PDO error [updating in table " . static::$table . "]: " . $e->getMessage() ); }

    }

    public static function order_by($order_by, $limit = 10){
        $db = DB::getConnection();

        $query = "SELECT " . static::generateAttributeList() . " FROM " . static::$table . " ORDER BY :order_by;";// LIMIT :limit";

        $st = $db->prepare($query);
        //echo $query ;
        try{
            $st->execute(array('order_by' => $order_by));    //, 'limit' => $limit));
        }
        catch( PDOException $e ) { exit( "PDO error [get top sorted elements " . static::$table . "]: " . $e->getMessage() ); }

        $class_name = get_called_class();
        $result = array();

        foreach($st->fetchAll() as $row)
            array_push($result, new $class_name($row) );
        
        return $result;
    }

}

?>