  const signature = await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [ourAccount],
    {
      commitment: 'recent'
    }
  )

  console.log(signature)