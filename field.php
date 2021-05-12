<?php

class Field{
	public $walls = array("left"=>false, "top"=>false, "right"=>false, "bottom"=>false);
	
	//if goal isn't null then it is an array with two 
	public $goal = NULL;   
	//robots will be represented with colors 
	public $robot = NULL; 

	function __construct($walls, $goal = NULL, $robot = NULL){
		if(strlen($walls) !== 4){
			echo "Eror while making field.";
			exit;
		}
		if($walls[0] === "1"){
			$this->walls["left"] = true;
		}

		if($walls[1] === "1")
			$this->walls["top"] = true;
		
		if($walls[2] === "1")
			$this->walls["right"] = true;
		
		if($walls[3] === "1")
			$this->walls["bottom"] = true;
		
		$this->goal = $goal;
		$this->robot = $robot;
	}

	function draw_field(){
		echo "<td class=\"board_field\" style = \"";
		if($this->robot != NULL)
			echo "background: " . $this->robot . ";";
		//treba dodati i za one ciljeve ali o tom po tom

		foreach($this->walls as $wall=>$exists){
			if ($exists)
				echo "border-" . $wall . ": 3px solid brown;";
		}

		echo " \">  </td>";
	}

};
?>
